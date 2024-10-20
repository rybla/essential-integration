use std::marker::PhantomData;

use abi::Resolution;
use anyhow::bail;
use essential_types::{Key, Value, Word};

pub mod init_oracle;

/// Represents a query result, which may or may not contain a value.
#[derive(Clone)]
pub struct Query<T>(pub Option<Value>, pub PhantomData<T>);

pub mod abi {
    pint_abi::gen_from_file! {
        abi: "../pint/out/debug/prediction-market-abi.json",
        contract:  "../pint/out/debug/prediction-market.json",
    }
}

/// Generates the key for querying a user's nonce.
pub fn user_nonce_key(hashed_key: [Word; 4]) -> Key {
    let keys: Vec<_> = abi::storage::keys::keys()
        .user_nonces(|e| e.entry(hashed_key))
        .into();
    keys.into_iter().next().expect("Must be a key")
}

/// Generates the key for querying an oracle's nonce.
pub fn oracle_nonce_key(hashed_key: [Word; 4]) -> Key {
    let keys: Vec<_> = abi::storage::keys::keys()
        .oracle_nonces(|e| e.entry(hashed_key))
        .into();
    keys.into_iter().next().expect("Must be a key")
}

/// Generates the key for querying an oracle's result.
pub fn oracle_result_key(hashed_key: [Word; 4]) -> Key {
    let keys: Vec<_> = abi::storage::keys::keys()
        .oracle_resolutions(|e| e.entry(hashed_key))
        .into();
    keys.into_iter().next().expect("Must be a key")
}

pub fn from_query_resolution(query: &Query<Resolution>) -> anyhow::Result<Resolution> {
    let r = match &query.0 {
        Some(resolution) => match &resolution[..] {
            [] => bail!("Expected resolution to be non-nil, got nil"),
            [0, _] => Resolution::Unresolved,
            [1, b] => Resolution::Resolved(*b != 0),
            _ => bail!("Expected two words, got: {:?}", resolution),
        },
        None => bail!("Expected query to be non-None, got None"),
    };
    Ok(r)
}

pub fn from_query_word(query: &Query<Word>) -> anyhow::Result<Word> {
    let r = match &query.0 {
        Some(nonce) => match &nonce[..] {
            [] => 0,
            [nonce] => *nonce,
            _ => bail!("Expected single word, got: {:?}", nonce),
        },
        None => 0,
    };
    Ok(r)
}

// CreateOracle
// TODO
