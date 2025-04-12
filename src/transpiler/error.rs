//! Error Module

use core::fmt::{Display, Formatter, Result};

use elf::{compression::CompressionHeader, ParseError};

/// Embive Transpiler Error
#[derive(Debug)]
pub enum Error {
    /// Error parsing ELF.
    ErrorParsingELF(ParseError),
    /// Section does not have a segment. The section index is provided.
    NoSegmentForSection(usize),
    /// Invalid instruction. The instruction is provided.
    InvalidInstruction(u32),
    /// Invalid instruction size. The size is provided.
    InvalidInstructionSize(usize),
    /// Invalid platform (not a RISC-V 32-bit ELF).
    InvalidPlatform,
    /// ELF has no section header table.
    NoSectionHeader,
    /// ELF has no program header table.
    NoProgramHeader,
    /// Buffer is too small.
    BufferTooSmall,
    /// Unsupported ELF Compression
    UnsupportedCompression(CompressionHeader),
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
