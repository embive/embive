//! RISC-V Instruction Macro module.

/// Macro for RISC-V Instructions
///
/// Arguments:
/// - `name`: Instruction name.
/// - `opcode`: Instruction opcode.
macro_rules! instruction {
    ($name:ident, $opcode:expr) => {
        /// Embive Instruction
        pub struct $name {}

        impl $name {
            /// Instruction Opcode
            pub const OPCODE: u8 = $opcode;
        }
    };
}

pub(super) use instruction;
