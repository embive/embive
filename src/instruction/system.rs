use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode};

/// System OpCode
/// Format: I-Type.
/// Action: Halt
pub struct System {
    _ty: TypeI,
}

impl Opcode for System {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            _ty: TypeI::from(data),
        })
    }
}

impl Instruction for System {
    fn execute(&self, _engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Halt the engine
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misc_mem() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let misc_mem = System {
            _ty: TypeI {
                rd: 0,
                rs1: 0,
                imm: 0,
                funct3: 0,
            },
        };

        let result = misc_mem.execute(&mut engine);
        assert_eq!(result, Ok(false));
    }
}