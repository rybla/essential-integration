use std::marker::PhantomData;

use essential_app_utils::{self as utils, compile::compile_pint_project, db::Dbs};
use essential_signer::Signature;
use essential_types::{contract::Contract, convert::word_4_from_u8_32, solution::Solution, Word};
use essential_wallet::Wallet;
use prediction_market::{
    abi::{MarketCondition, Resolution},
    Query,
};

const ORACLE1_PRIVATE_KEY: &str =
    "128A3D2146A69581FD8FC4C0A9B7A96A5755D85255D4E47F814AFA69D7726C8D";
const ORACLE1_NAME: &str = "oracle1";

const MARKET1_PRIVATE_KEY: &str =
    "228A3D2146A69581FD8FC4C0A9B7A96A5755D85255D4E47F814AFA69D7726C8D";
const MARKET1_NAME: &str = "market1";

#[tokio::test]
async fn test_create_oracle() {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create new databases for testing
    let dbs = utils::db::new_dbs().await;

    let contract = compile_pint_project(concat!(env!("CARGO_MANIFEST_DIR"), "/../pint").into())
        .await
        .unwrap();

    // deploy the token contract
    essential_app_utils::deploy::deploy_contract(&dbs.builder, &contract)
        .await
        .unwrap();

    // temporary wallet for testing
    let mut wallet = essential_wallet::Wallet::temp().unwrap();

    // setup Oracle account
    let oracle_key = hex::decode(ORACLE1_PRIVATE_KEY).unwrap();
    wallet
        .insert_key(
            ORACLE1_NAME,
            essential_signer::Key::Secp256k1(
                essential_signer::secp256k1::SecretKey::from_slice(&oracle_key).unwrap(),
            ),
        )
        .unwrap();
    let oracle_hashed_key = hash_key(&mut wallet, ORACLE1_NAME);

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

    let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
    let oracle_resolution_query: Query<Word> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_resolution_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );
    let oracle_resolution = prediction_market::from_query_word(&oracle_resolution_query).unwrap();

    let init = prediction_market::init_oracle::Init {
        oracle_hashed_key,
        oracle_nonce_query: oracle_nonce_query.clone(),
    };
    let to_sign = prediction_market::init_oracle::data_to_sign(init).unwrap();
    let Signature::Secp256k1(signature) = wallet
        .sign_words(&to_sign.to_words(), ORACLE1_NAME)
        .unwrap()
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

    // assert that oracle was initialized properly
    let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
    let oracle_resolution_query: Query<prediction_market::abi::Resolution> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_resolution_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );
    assert_eq!(
        prediction_market::from_query_resolution(&oracle_resolution_query).unwrap(),
        prediction_market::abi::Resolution::Unresolved
    );
}

#[tokio::test]
async fn test_create_oracle_and_market() {
    // Initialize tracing for better debugging

    // Create new databases for testing
    let dbs = utils::db::new_dbs().await;

    let contract = compile_pint_project(concat!(env!("CARGO_MANIFEST_DIR"), "/../pint").into())
        .await
        .unwrap();

    // deploy the token contract
    essential_app_utils::deploy::deploy_contract(&dbs.builder, &contract)
        .await
        .unwrap();

    // temporary wallet for testing
    let mut wallet = essential_wallet::Wallet::temp().unwrap();

    // setup oracle account
    let oracle_key = hex::decode(ORACLE1_PRIVATE_KEY).unwrap();
    wallet
        .insert_key(
            ORACLE1_NAME,
            essential_signer::Key::Secp256k1(
                essential_signer::secp256k1::SecretKey::from_slice(&oracle_key).unwrap(),
            ),
        )
        .unwrap();
    let oracle_hashed_key = hash_key(&mut wallet, ORACLE1_NAME);

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

    // let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
    // let oracle_resolution_query: Query<Word> = Query(
    //     utils::node::query_state_head(
    //         &dbs.node,
    //         &prediction_market::abi::ADDRESS,
    //         &oracle_resolution_key,
    //     )
    //     .await
    //     .unwrap(),
    //     PhantomData,
    // );
    // let oracle_resolution = prediction_market::from_query_word(&oracle_resolution_query).unwrap();

    let solution = {
        let init = prediction_market::init_oracle::Init {
            oracle_hashed_key,
            oracle_nonce_query: oracle_nonce_query.clone(),
        };
        let to_sign = prediction_market::init_oracle::data_to_sign(init).unwrap();
        let Signature::Secp256k1(signature) = wallet
            .sign_words(&to_sign.to_words(), ORACLE1_NAME)
            .unwrap()
        else {
            panic!("invalid signature")
        };

        // construct solution
        let solution = prediction_market::init_oracle::build_solution(
            prediction_market::init_oracle::BuildSolution {
                oracle_hashed_key,
                // new_oracle_nonce: prediction_market::from_query_word(&oracle_nonce_query).unwrap() + 1,
                new_oracle_nonce: 1,
                new_oracle_resolution: prediction_market::abi::Resolution::Unresolved,
                signature,
            },
        )
        .unwrap();
        solution
    };

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

    // now, init a market that refers to the oracle

    // setup market account
    let market_key = hex::decode(MARKET1_PRIVATE_KEY).unwrap();
    wallet
        .insert_key(
            MARKET1_NAME,
            essential_signer::Key::Secp256k1(
                essential_signer::secp256k1::SecretKey::from_slice(&market_key).unwrap(),
            ),
        )
        .unwrap();
    let market_hashed_key = hash_key(&mut wallet, MARKET1_NAME);

    let market_nonce_key = prediction_market::market_nonce_key(market_hashed_key);
    let market_nonce_query: Query<Word> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &market_nonce_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );

    // let market_condition_key = prediction_market::market_condition_key(market_hashed_key);
    // let market_condition_query: Query<Word> = Query(
    //     utils::node::query_state_head(
    //         &dbs.node,
    //         &prediction_market::abi::ADDRESS,
    //         &market_condition_key,
    //     )
    //     .await
    //     .unwrap(),
    //     PhantomData,
    // );

    let solution = {
        let init = prediction_market::init_market::Init {
            market_hashed_key,
            new_market_condition: MarketCondition::Oracle(oracle_hashed_key),
            market_nonce_query: market_nonce_query.clone(),
        };

        let to_sign = prediction_market::init_market::data_to_sign(init).unwrap();
        let Signature::Secp256k1(signature) = wallet
            .sign_words(&to_sign.to_words(), MARKET1_NAME)
            .unwrap()
        else {
            panic!("invalid signature")
        };

        // construct solution
        let solution = prediction_market::init_market::build_solution(
            prediction_market::init_market::BuildSolution {
                new_market_nonce: 1,
                market_hashed_key,
                new_market_condition: MarketCondition::Oracle(oracle_hashed_key),
                signature,
            },
        )
        .unwrap();

        solution
    };

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
}

#[tokio::test]
async fn test_create_and_resolve_oracle() {
    // parameters
    let oracle_private_key = "128A3D2146A69581FD8FC4C0A9B7A96A5755D85255D4E47F814AFA69D7726C8D";
    let oracle_name = "my_oracle";

    // Create new databases for testing
    let dbs = utils::db::new_dbs().await;

    let contract = compile_pint_project(concat!(env!("CARGO_MANIFEST_DIR"), "/../pint").into())
        .await
        .unwrap();

    // deploy the token contract
    essential_app_utils::deploy::deploy_contract(&dbs.builder, &contract)
        .await
        .unwrap();

    // temporary wallet for testing
    let mut wallet = essential_wallet::Wallet::temp().unwrap();
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

    // InitOracle
    let solution = {
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

        let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
        let oracle_resolution_query: Query<Word> = Query(
            utils::node::query_state_head(
                &dbs.node,
                &prediction_market::abi::ADDRESS,
                &oracle_resolution_key,
            )
            .await
            .unwrap(),
            PhantomData,
        );
        let oracle_resolution =
            prediction_market::from_query_word(&oracle_resolution_query).unwrap();

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
        prediction_market::init_oracle::build_solution(
            prediction_market::init_oracle::BuildSolution {
                oracle_hashed_key,
                new_oracle_nonce: prediction_market::from_query_word(&oracle_nonce_query).unwrap()
                    + 1,
                new_oracle_resolution: prediction_market::abi::Resolution::Unresolved,
                signature,
            },
        )
        .unwrap()
    };

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

    // ResolveOracle
    let solution = {
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

        let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
        let oracle_resolution_query: Query<Resolution> = Query(
            utils::node::query_state_head(
                &dbs.node,
                &prediction_market::abi::ADDRESS,
                &oracle_resolution_key,
            )
            .await
            .unwrap(),
            PhantomData,
        );
        let oracle_resolution =
            prediction_market::from_query_resolution(&oracle_resolution_query).unwrap();

        let init = prediction_market::resolve_oracle::Init {
            oracle_hashed_key,
            oracle_nonce_query: oracle_nonce_query.clone(),
            new_resolution: true,
        };
        let to_sign = prediction_market::resolve_oracle::data_to_sign(init).unwrap();
        let Signature::Secp256k1(signature) =
            wallet.sign_words(&to_sign.to_words(), oracle_name).unwrap()
        else {
            panic!("invalid signature")
        };

        // construct solution
        prediction_market::resolve_oracle::build_solution(
            prediction_market::resolve_oracle::BuildSolution {
                oracle_hashed_key,
                new_oracle_nonce: prediction_market::from_query_word(&oracle_nonce_query).unwrap()
                    + 1,
                signature,
                new_resolution: true,
            },
        )
        .unwrap()
    };

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

    // assert that oracle was resolved properly
    let oracle_resolution_key = prediction_market::oracle_resolution_key(oracle_hashed_key);
    let oracle_resolution_query: Query<prediction_market::abi::Resolution> = Query(
        utils::node::query_state_head(
            &dbs.node,
            &prediction_market::abi::ADDRESS,
            &oracle_resolution_key,
        )
        .await
        .unwrap(),
        PhantomData,
    );
    assert_eq!(
        prediction_market::from_query_resolution(&oracle_resolution_query).unwrap(),
        prediction_market::abi::Resolution::Resolved(true)
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
