//! Error Module

use core::fmt::{Display, Formatter, Result};

/// Embive Error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Memory address is out of bounds.
    InvalidMemoryAddress,
    /// Program counter is out of bounds.
    InvalidProgramCounter,
    /// Instruction is not implemented.
    InvalidInstruction,
    /// CPU Register is out of bounds.
    InvalidCPURegister,
    /// Control and Status Register is invalid or not supported.
    InvalidCSRegister,
    /// No syscall function is set.
    NoSyscallFunction,
    /// Custom error.
    Custom(&'static str),
}

impl core::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}
