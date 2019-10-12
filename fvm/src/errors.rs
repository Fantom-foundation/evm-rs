//! This module contains errors related to the Fantom VM itself
use failure::Error;

/// Convenience wrapper around T and a VMError
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Fail)]
/// Errors related to the VM
pub enum VMError {
    // VM has encountered an unknown opcode
    #[fail(display = "an unrecognized opcode was found")]
    UnknownOpcodeError,
    // VM has run out of memory
    #[fail(display = "out of memory")]
    MemoryError,
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
