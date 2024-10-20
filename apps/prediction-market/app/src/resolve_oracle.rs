use essential_app_utils::inputs::Encode;
use essential_sign::secp256k1::ecdsa::RecoverableSignature;
use essential_types::{
    solution::{Solution, SolutionData},
    Word,
};

use crate::{from_query_word, Query};

pub struct Init {
    pub oracle_hashed_key: [Word; 4],
    pub oracle_nonce_query: Query<Word>,
    pub new_resolution: bool,
}

pub struct BuildSolution {
    pub oracle_hashed_key: [Word; 4],
    pub new_oracle_nonce: Word,
    pub new_resolution: bool,
    pub signature: RecoverableSignature,
}

pub fn build_solution(
    BuildSolution {
        oracle_hashed_key,
        new_oracle_nonce,
        new_resolution,
        signature,
    }: BuildSolution,
) -> anyhow::Result<Solution> {
    let pub_vars = super::abi::ResolveOracle::PubVars {
        key: oracle_hashed_key,
        new_resolution,
    };
    let vars = super::abi::ResolveOracle::Vars {
        sig: signature.encode(),
    };
    let muts = super::abi::storage::mutations()
        .oracle_nonces(|nonces| nonces.entry(oracle_hashed_key, new_oracle_nonce))
        .oracle_resolutions(|oracle_resolutions| {
            oracle_resolutions.entry(
                oracle_hashed_key,
                super::abi::Resolution::Resolved(new_resolution),
            )
        });
    let solution = SolutionData {
        predicate_to_solve: super::abi::ResolveOracle::ADDRESS,
        decision_variables: vars.into(),
        transient_data: pub_vars.into(),
        state_mutations: muts.into(),
    };
    Ok(Solution {
        data: vec![solution],
    })
}

// ----------------------------------------------------------------------------
// signing
// ----------------------------------------------------------------------------

pub struct ToSign {
    pub oracle_hashed_key: [Word; 4],
    pub new_oracle_nonce: Word,
    pub new_resolution: bool,
}

impl ToSign {
    /// Converts the ToSign struct to a vector of Words for signing.
    pub fn to_words(&self) -> Vec<Word> {
        vec![
            self.oracle_hashed_key[0],
            self.oracle_hashed_key[1],
            self.oracle_hashed_key[2],
            self.oracle_hashed_key[3],
            self.new_resolution as i64,
            self.new_oracle_nonce,
        ]
    }
}

pub fn data_to_sign(
    Init {
        oracle_hashed_key,
        oracle_nonce_query,
        new_resolution,
    }: Init,
) -> anyhow::Result<ToSign> {
    let new_oracle_nonce = from_query_word(&oracle_nonce_query)? + 1;
    Ok(ToSign {
        oracle_hashed_key,
        new_oracle_nonce,
        new_resolution,
    })
}
