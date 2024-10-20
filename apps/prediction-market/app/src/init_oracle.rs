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
}

pub struct BuildSolution {
    pub oracle_hashed_key: [Word; 4],
    pub new_oracle_nonce: Word,
    pub new_oracle_resolution: super::abi::Resolution,
    /// The signature over the data.
    pub signature: RecoverableSignature,
}

pub fn build_solution(
    BuildSolution {
        new_oracle_nonce,
        oracle_hashed_key,
        new_oracle_resolution,
        signature,
    }: BuildSolution,
) -> anyhow::Result<Solution> {
    let pub_vars = super::abi::InitOracle::PubVars {
        key: oracle_hashed_key,
    };
    let vars = super::abi::InitOracle::Vars {
        sig: signature.encode(),
    };
    let muts = super::abi::storage::mutations()
        .oracle_nonces(|nonces| nonces.entry(oracle_hashed_key, new_oracle_nonce))
        .oracle_resolutions(|oracle_resolutions| {
            oracle_resolutions.entry(oracle_hashed_key, new_oracle_resolution)
        });
    let solution = SolutionData {
        predicate_to_solve: super::abi::InitOracle::ADDRESS,
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

/// Represents the data to be signed for a mint solution.
pub struct ToSign {
    pub oracle_hashed_key: [Word; 4],
    pub new_oracle_nonce: Word,
}

impl ToSign {
    /// Converts the ToSign struct to a vector of Words for signing.
    pub fn to_words(&self) -> Vec<Word> {
        vec![
            self.oracle_hashed_key[0],
            self.oracle_hashed_key[1],
            self.oracle_hashed_key[2],
            self.oracle_hashed_key[3],
            self.new_oracle_nonce,
        ]
    }
}

/// prepare the data to be signed for an init_oracle transaction
pub fn data_to_sign(
    Init {
        oracle_hashed_key,
        oracle_nonce_query: oracle_nonce,
    }: Init,
) -> anyhow::Result<ToSign> {
    let new_oracle_nonce = from_query_word(&oracle_nonce)? + 1;
    Ok(ToSign {
        oracle_hashed_key,
        new_oracle_nonce,
    })
}
