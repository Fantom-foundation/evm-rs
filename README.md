evm-rs
------
[![Rust: nightly](https://img.shields.io/badge/Rust-nightly-blue.svg)](https://www.rust-lang.org) [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE) [![Build Status](https://travis-ci.org/Fantom-foundation/evm-rs.svg?branch=master)](https://travis-ci.org/Fantom-foundation/evm-rs)

EVM implementation in Rust. Stack-based.

This contains our Ethereum Virtual Machine. Any valid Solidity bytecode should run on the EVM.

---

## RFCs

https://github.com/Fantom-foundation/fantom-rfcs

# Developer guide

Install the latest version of [Rust](https://www.rust-lang.org). We tend to use nightly versions. [CLI tool for installing Rust](https://rustup.rs).

We use [rust-clippy](https://github.com/rust-lang-nursery/rust-clippy) linters to improve code quality.

There are plenty of [IDEs](https://areweideyet.com) and other [Rust development tools to consider](https://github.com/rust-unofficial/awesome-rust#development-tools).

### CLI instructions

```bash
# Install Rust (nightly)
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly
# Install cargo-make (cross-platform feature-rich reimplementation of Make)
$ cargo install --force cargo-make
# Install rustfmt (Rust formatter)
$ rustup component add rustfmt
# Install clippy (Rust linter)
$ rustup component add clippy
# Clone this repo
$ git clone https://github.com/Fantom-foundation/evm-rs && cd evm-rs
# Run tests
$ cargo test
# Format, build and test
$ cargo make
```
