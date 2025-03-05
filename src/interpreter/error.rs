//! Interpreter Error Module

use core::fmt::{Display, Formatter, Result};

/// Embive Error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Memory address is out of bounds. The memory address is provided.
    InvalidMemoryAddress(u32),
    /// Program counter is out of bounds. The program counter is provided.
    InvalidProgramCounter(u32),
    /// Instruction is invalid. The program counter is provided.
    InvalidInstruction(u32),
    /// Control and Status Register is invalid or not supported. The CSR address is provided.
    InvalidCSRegister(u16),
    /// CPU Register is out of bounds. The register index is provided.
    InvalidCPURegister(u8),
    /// Instruction is illegal. The program counter is provided.
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
