use core::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum EmbiveError {
    InvalidMemoryAddress,
    InvalidProgramCounter,
    InvalidInstruction,
    InvalidRegister,
}

impl Error for EmbiveError {}

impl Display for EmbiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
