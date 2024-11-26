//! Error Module

use core::{error::Error, fmt::Display};

/// Embive Error Enum
#[derive(Debug, PartialEq)]
pub enum EmbiveError {
    /// Memory address is out of bounds.
    InvalidMemoryAddress,
    /// Program counter is out of bounds.
    InvalidProgramCounter,
    /// Instruction is not implemented.
    InvalidInstruction,
    /// Register is out of bounds.
    InvalidRegister,
    /// No syscall function is set.
    NoSyscallFunction,
    /// Custom error.
    Custom(&'static str),
}

impl Error for EmbiveError {}

impl Display for EmbiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
