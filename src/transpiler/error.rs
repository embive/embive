//! Error Module

use core::fmt::{Display, Formatter, Result};

use elf::ParseError;

/// Embive Error
#[derive(Debug)]
pub enum Error {
    /// Error parsing ELF.
    ErrorParsingELF(ParseError),
    /// Section does not have a segment. The section index is provided.
    NoSegmentForSection(usize),
    /// Missing section data in the ELF. The section index is provided.
    MissingSectionData(usize),
    /// Invalid instruction. The instruction is provided.
    InvalidInstruction(u32),
    /// Invalid platform (not a RISC-V 32-bit ELF).
    InvalidPlatform,
    /// ELF has no section header table.
    NoSectionHeader,
    /// ELF has no program header table.
    NoProgramHeader,
    /// Buffer is too small.
    BufferTooSmall,
}

impl core::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::ErrorParsingELF(e)
    }
}
