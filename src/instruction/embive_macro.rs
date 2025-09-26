//! Embive Instruction Macro module.

/// Macro for Embive Instruction
macro_rules! instruction {
    ($name:ident, $opcode:expr, $format:ty, {$($cty:ty: {$($cname:ident = $cvalue:expr);* $(;)?}),* $(,)?}) => {
        impl $name {
            $(
                $(
                    /// Instruction Constant Value
                    pub const $cname: $cty = $cvalue;
                )*
            )*
        }
        crate::instruction::embive_macro::instruction!($name, $opcode, $format);
    };
    ($name:ident, $opcode:expr, $format:ty) => {
        /// Embive Instruction
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name (pub $format);

        impl crate::instruction::embive::InstructionImpl for $name {
            /// Instruction Opcode
            #[inline(always)]
            fn opcode() -> u8 {
                $opcode
            }

            /// Instruction size
            #[inline(always)]
            fn size() -> crate::format::Size {
                <$format>::SIZE
            }

            /// Encode instruction to u32 (Embive Format)
            #[inline(always)]
            fn encode(&self) -> u32 {
                self.0.to_embive()
            }

            /// Decode instruction from u32 (Embive Format)
            #[inline(always)]
            fn decode(inst: u32) -> Self {
                Self(<$format>::from_embive(inst))
            }
        }

        impl From<$format> for $name {
            fn from(format: $format) -> Self {
                Self(format)
            }
        }
    };
}

/// Macro for Embive Instructions
macro_rules! instructions {
    {$($opcode:expr => $name:ident: $format:ty = $consts:tt);* $(;)?} => {
        $(
            crate::instruction::embive_macro::instruction!($name, $opcode, $format, $consts);
        )*

        /// Embive Instruction Decoding Macro
        macro_rules! decode_instruction {
            ($inst:expr, $method:tt, $params:tt) => {
                {
                    use crate::instruction::embive::InstructionImpl;
                    let inst = u32::from($inst);

                    match (inst & 0x1F) {
                        $(
                            $opcode => Some(crate::instruction::embive::$name::decode(inst).$method$params),
                        )*
                        _ => None,
                    }
                }
            };
        }

        pub(crate) use decode_instruction;
    };
}

pub(super) use instruction;
pub(super) use instructions;
