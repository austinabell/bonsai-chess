use std::io::Read;

use alloy_sol_types::{token::TokenSeq, SolType};
use derisc0::{FromParameter, IntoResponse};
use risc0_zkvm::guest::env;

// Re-export sol macro for convenience.
pub use alloy_sol_types::sol;

/// Wrapper type to handle Ethereum abi encoding and decoding of parameters through the risc0
/// VM.
///
/// ```no_run
/// use risc0_alloy::{EthParams, sol};
///
/// derisc0::entry!(some_function);
///
/// fn some_function(EthParams((a,)): EthParams<sol!(tuple(uint32,))>) -> Result<EthParams<sol!(tuple(uint32,))>, &'static str> {
///     Ok(EthParams((a.checked_mul(a).ok_or("integer overflow")?,)))
/// }
/// # fn main() {
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct EthParams<T: SolType>(pub T::RustType);

impl<T> FromParameter for EthParams<T>
where
    T: SolType,
    for<'de> T::TokenType<'de>: TokenSeq<'de>,
{
    fn from_parameter() -> Self {
        // Read data sent from the application contract.
        let mut input_bytes = Vec::<u8>::new();
        env::stdin().read_to_end(&mut input_bytes).unwrap();

        // Decode parameters from the scheduled call on eth.
        let val = T::decode_params(&input_bytes, true).unwrap();
        Self(val)
    }
}

impl<T> IntoResponse for EthParams<T>
where
    T: SolType,
    for<'de> T::TokenType<'de>: TokenSeq<'de>,
{
    fn handle_response(self) {
        env::commit_slice(&T::encode(&self.0));
    }
}
