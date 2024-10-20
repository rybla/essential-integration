use std::marker::PhantomData;

use essential_app_utils::{self as utils, compile::compile_pint_project};
use essential_signer::Signature;
use essential_types::{convert::word_4_from_u8_32, Word};
use essential_wallet::Wallet;
use prediction_market::Query;
// use token::Query;

#[tokio::test]
async fn test_create_oracle() {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // parameters
    let oracle_private_key = "128A3D2146A69581FD8FC4C0A9B7A96A5755D85255D4E47F814AFA69D7726C8D";
    let oracle_name = "my_oracle";

    // Create new databases for testing
    let dbs = utils::db::new_dbs().await;

    let transfer = compile_pint_project(concat!(env!("CARGO_MANIFEST_DIR"), "/../pint").into())
        .await
        .unwrap();

    // deploy the token contract
    essential_app_utils::deploy::deploy_contract(&dbs.builder, &transfer)
        .await
        .unwrap();

    // temporary wallet for testing
    let mut wallet = essential_wallet::Wallet::temp().unwrap();

    // setup Oracle account
    let oracle_key = hex::decode(oracle_private_key).unwrap();
    wallet
        .insert_key(
            oracle_name,
            essential_signer::Key::Secp256k1(
                essential_signer::secp256k1::SecretKey::from_slice(&oracle_key).unwrap(),
            ),
        )
        .unwrap();
    let oracle_hashed_key = hash_key(&mut wallet, oracle_name);

    let oracle_nonce_key = prediction_market::oracle_nonce_key(oracle_hashed_key);
    let oracle_nonce_query: Query<Word> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_nonce_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );

    let oracle_result_key = prediction_market::oracle_result_key(oracle_hashed_key);
    let oracle_result_query: Query<Word> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_result_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );
    let oracle_result = prediction_market::from_query_word(&oracle_result_query).unwrap();

    let init = prediction_market::init_oracle::Init {
        oracle_hashed_key,
        oracle_nonce_query: oracle_nonce_query.clone(),
    };
    let to_sign = prediction_market::init_oracle::data_to_sign(init).unwrap();
    let Signature::Secp256k1(signature) =
        wallet.sign_words(&to_sign.to_words(), oracle_name).unwrap()
    else {
        panic!("invalid signature")
    };

    // construct solution
    let solution = prediction_market::init_oracle::build_solution(
        prediction_market::init_oracle::BuildSolution {
            oracle_hashed_key,
            new_oracle_nonce: prediction_market::from_query_word(&oracle_nonce_query).unwrap() + 1,
            new_oracle_resolution: prediction_market::abi::Resolution::Unresolved,
            signature,
        },
    )
    .unwrap();

    // submit the solution
    utils::builder::submit(&dbs.builder, solution.clone())
        .await
        .unwrap();

    // validate the solution
    utils::node::validate_solution(&dbs.node, solution.clone())
        .await
        .unwrap();

    // Build a block
    {
        let o = utils::builder::build_default(&dbs).await.unwrap();
        assert!(o.failed.is_empty(), "{:?}", o.failed);
    }

    // verify oracle exists
    let oracle_result_key = prediction_market::oracle_result_key(oracle_hashed_key);
    let oracle_result_query: Query<prediction_market::abi::Resolution> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_result_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );
    assert_eq!(
        prediction_market::from_query_resolution(&oracle_result_query).unwrap(),
        prediction_market::abi::Resolution::Unresolved
    );
}

// Helper function to hash a public key
fn hash_key(wallet: &mut Wallet, account_name: &str) -> [Word; 4] {
    let public_key = wallet.get_public_key(account_name).unwrap();
    let essential_signer::PublicKey::Secp256k1(public_key) = public_key else {
        panic!("Invalid public key")
    };
    let encoded = essential_sign::encode::public_key(&public_key);
    word_4_from_u8_32(essential_hash::hash_words(&encoded))
}
