evm-rs
------
[![Rust: nightly](https://img.shields.io/badge/Rust-nightly-blue.svg)](https://www.rust-lang.org) [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE) [![Build Status](https://travis-ci.org/Fantom-foundation/evm-rs.svg?branch=master)](https://travis-ci.org/Fantom-foundation/evm-rs)

This is a register-based virtual machine intended to execute transactions for the Fantom cryptocurrency. In its current state, it can function as an Ethereum VM and full node.

## Consensus Algorithms

By default, the FVM utilizes the ethash consensus algorithm used by Ethereum. The goal is for it to use the Lachesis protocol.

## Structure

This project is structured as a Cargo workspace, with three sub-projects: `client`, `fvm`, and `world`.

### Client

This provides the CLI binary, and is responsible for setting up the various servers over which the FVM will accept requests. It also contains utilities for managing keys and accounts.

#### CLI Options

These are defined in `client/src/cli.yml`, and uses the Rust `clap` library.

### FVM

This contains the Fantom Virtual Machine. It implements the Ethereum Virtual Machine, but emulates registers rather than a stack. Any valid Solidity bytecode should run on the FVM.

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
$ git clone https://github.com/Fantom-foundation/light-cli-rs && cd light-cli-rs
# Run tests
$ cargo test
# Format, build and test
$ cargo make
```
