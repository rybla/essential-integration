use anyhow::bail;
use essential_types::{Key, Value, Word};

/// Module containing the prediction-market contract ABI.
#[allow(missing_docs)]
pub mod token {
    pint_abi::gen_from_file! {
        abi: "../pint/out/debug/prediction-market-abi.json",
        contract:  "../pint/out/debug/prediction-market.json",
    }
}
