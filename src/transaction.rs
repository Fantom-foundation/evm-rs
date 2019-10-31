use ethereum_types::{H160, U256};
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Serialize, Hash, Eq, PartialOrd, Ord)]
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

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
