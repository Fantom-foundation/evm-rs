//! Contains the Transaction module

use ethereum_types::{H160, U256};
pub mod pool;

/// Core data structure for interacting with the EVM
#[derive(Debug, Default, Deserialize, Clone, PartialEq, Serialize)]
pub struct Transaction {
    /// Nonce
    pub nonce: U256,
    /// Gas Price
    pub gas_price: U256,
    /// Start Gas
    pub start_gas: U256,
    /// Recipient
    /// If None, then this is a contract creation
    pub to: Option<H160>,
    /// Transferred value
    pub value: U256,
    /// Data
    pub data: Vec<u8>,
    /// The standardised V field of the signature.
    pub v: U256,
    /// The R field of the signature.
    pub r: U256,
    /// The S field of the signature.
    pub s: U256,
}

/// A valid transaction is one where:
/// (i) the signature is well-formed (ie. 0 <= v <= 3, 0 <= r < P, 0 <= s < N, 0 <= r < P - N if v >= 2),
/// and (ii) the sending account has enough funds to pay the fee and the value.
impl Transaction {
    pub fn is_valid(&self) -> bool {
        unimplemented!()
    }

    fn sender_account(&mut self) {
        unimplemented!()
    }
}
