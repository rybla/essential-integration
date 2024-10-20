use essential_app_utils::inputs::Encode;
use essential_sign::secp256k1::ecdsa::RecoverableSignature;
use essential_types::{
    solution::{Solution, SolutionData},
    Word,
};

use crate::{abi::MarketCondition, from_query_word, HashedKey, Query};

pub struct Init {
    pub market_hashed_key: HashedKey,
    pub new_market_condition: crate::abi::MarketCondition,
    pub market_nonce_query: Query<Word>,
}

pub struct BuildSolution {
    pub market_hashed_key: HashedKey,
    pub new_market_condition: crate::abi::MarketCondition,
    pub signature: RecoverableSignature,
    pub new_market_nonce: Word,
}

pub fn build_solution(
    BuildSolution {
        market_hashed_key,
        new_market_condition,
        signature,
        new_market_nonce,
    }: BuildSolution,
) -> anyhow::Result<Solution> {
    let pub_vars = crate::abi::InitMarket::PubVars {
        key: market_hashed_key,
        new_condition: new_market_condition.clone(),
    };
    let vars = crate::abi::InitMarket::Vars {
        sig: signature.encode(),
    };
    let muts = crate::abi::storage::mutations()
        .market_nonces(|m| m.entry(market_hashed_key, new_market_nonce))
        .market_resolutions(|m| m.entry(market_hashed_key, crate::abi::Resolution::Unresolved))
        .market_conditions(|m| m.entry(market_hashed_key, new_market_condition));
    let solution = SolutionData {
        predicate_to_solve: crate::abi::InitMarket::ADDRESS,
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
    pub market_hashed_key: HashedKey,
    pub new_market_condition: MarketCondition,
    pub new_market_nonce: Word,
}

impl ToSign {
    /// Converts the ToSign struct to a vector of Words for signing.
    pub fn to_words(&self) -> Vec<Word> {
        vec![
            self.market_hashed_key[0],
            self.market_hashed_key[1],
            self.market_hashed_key[2],
            self.market_hashed_key[3],
            // self.new_market_condition // ΤΟDO: encode as Words,
            self.new_market_nonce,
        ]
    }
}

pub fn data_to_sign(
    Init {
        market_hashed_key,
        market_nonce_query,
        new_market_condition,
    }: Init,
) -> anyhow::Result<ToSign> {
    let new_market_nonce = from_query_word(&market_nonce_query)? + 1;
    Ok(ToSign {
        market_hashed_key,
        new_market_nonce,
        new_market_condition,
    })
}
