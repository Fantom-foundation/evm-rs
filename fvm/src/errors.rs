//! This module contains errors related to the Fantom VM itself
use failure::Error;

/// Convenience wrapper around T and a VMError
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Fail)]
/// Errors related to the VM
pub enum VMError {
    #[fail(display = "an unrecognized opcode was found")]
    UnknownOpcodeError,
    #[fail(display = "out of memory")]
    MemoryError,
    #[fail(display = "Invalid instruction")]
    InvalidInstruction,
}

#[derive(Debug, Clone, Fail)]
/// Errors related to Storage
pub enum StorageError {
    #[fail(display = "commit area")]
    CommitError,
    #[fail(display = "require area")]
    RequireError,
    #[fail(display = "invalid commitment")]
    InvalidCommitment,
    #[fail(display = "already committed")]
    AlreadyCommitted,
}
