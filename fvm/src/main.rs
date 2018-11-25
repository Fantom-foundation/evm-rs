extern crate bigint;
extern crate trie;

extern crate log;
extern crate env_logger;

mod memory;
mod account;
mod opcodes;
mod storage;
mod vm;
mod errors;

fn main() {
    env_logger::init();
    println!("Hello!");
}