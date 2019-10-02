//! Holds a pool of transactions

use futures::stream::Stream;
use futures::Poll;
use futures::task::Context;
use libconsensus::{Consensus, ConsensusConfiguration};
use libconsensus::errors::Result;
use transactions::Transaction;
use secp256k1::ContextFlag;
use std::pin::Pin;

pub struct TransactionPool {
    transactions: Vec<Transaction>
}

pub struct EthashConfiguration;

impl ConsensusConfiguration<Transaction> for EthashConfiguration {
    fn new() -> EthashConfiguration {
        EthashConfiguration
    }
}

impl Stream for TransactionPool {
    type Item = Transaction;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl Drop for TransactionPool {
    fn drop(&mut self) {
    }
}

impl<'a> Consensus<'a, Transaction> for TransactionPool {
    type Configuration = EthashConfiguration;

    fn new(cfg: Self::Configuration) -> Result<TransactionPool> {
        Ok(TransactionPool { transactions: vec![] })
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn send_transaction(&mut self, d: Transaction) -> Result<()> {
        Ok(())
    }
}