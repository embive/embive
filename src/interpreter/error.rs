//! Interpreter Error Module

use core::fmt::{Display, Formatter, Result};

/// Embive Error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Memory address is out of bounds.
    InvalidMemoryAddress(u32),
    /// Program counter is out of bounds.
    InvalidProgramCounter(u32),
    /// Instruction is invalid. The instruction is provided.
    InvalidInstruction(u32),
    /// Control and Status Register is invalid or not supported.
    InvalidCSRegister(u16),
    /// CPU Register is out of bounds.
    InvalidCPURegister(u8),
    /// Instruction is illegal. The instruction is provided.
    IllegalInstruction(u32),
    /// Interrupt not enabled by interpreted code (CSR `mie` bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`]).
    InterruptNotEnabled,
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
