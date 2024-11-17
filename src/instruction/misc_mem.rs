use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

/// Miscellaneous Memory OpCode
/// Format: I-Type.
/// Action: Nothing (Not implemented / Not applicable)
pub struct MiscMem {
    _ty: TypeI,
}

impl Opcode for MiscMem {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction {
        Self {
            _ty: TypeI::from(data),
        }
    }
}

impl Instruction for MiscMem {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Fencing isn't applicable to this implementation.
        // This is a nop.

        // Go to next instruction
        engine.pc = engine.pc.wrapping_add(INSTRUCTION_SIZE);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misc_mem() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.pc = 0x1;
        let misc_mem = MiscMem {
            _ty: TypeI {
                rd: 0,
                rs1: 0,
                imm: 0,
                funct3: 0,
            },
        };

        let result = misc_mem.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.pc, 0x1 + INSTRUCTION_SIZE);
    }
}
