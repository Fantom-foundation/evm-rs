//! Holds a pool of transactions
use std::pin::Pin;

use futures::stream::Stream;
use futures::task::Context;
use futures::Poll;
use libconsensus::errors::Result;
use libconsensus::{Consensus, ConsensusConfiguration};
use transactions::Transaction;

pub struct TransactionPool {
    transactions: Vec<Transaction>,
}

pub struct EthashConfiguration;

impl ConsensusConfiguration<Transaction> for EthashConfiguration {
    fn new() -> EthashConfiguration {
        EthashConfiguration
    }
}

impl Stream for TransactionPool {
    type Item = Transaction;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl Drop for TransactionPool {
    fn drop(&mut self) {}
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
