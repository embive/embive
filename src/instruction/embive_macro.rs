//! Embive Instruction Macro module.

/// Macro for Embive instructions
/// Arguments:
/// - `name`: Instruction name.
/// - `opcode`: Instruction opcode.
/// - `size`: Instruction size in bytes.
/// - `format`: Instruction format.
/// - `custom`: Instruction specific code/data.
macro_rules! instruction {
    ($name:ident, $opcode:expr, $size:expr, $format:ty, $custom:tt) => {
        /// Embive Instruction
        pub struct $name {}

        impl $name {
            /// Instruction Opcode
            pub const OPCODE: u8 = $opcode;

            /// Instruction size in bytes
            pub const SIZE: crate::instruction::Size = $size;

            /// Decode instruction from u32 (Embive Format)
            #[cfg(feature = "interpreter")]
            #[inline(always)]
            pub fn decode(inst: u32) -> $format {
                <$format>::from_embive(inst)
            }

            /// Encode instruction to u32 (Embive Format)
            #[cfg(feature = "transpiler")]
            #[inline(always)]
            pub fn encode(inst: $format) -> u32 {
                inst.to_embive()
            }
        }

        impl $name $custom
    };
}

pub(super) use instruction;
