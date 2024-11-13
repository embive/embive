use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeU;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

/// Load Upper Immediate
/// Both an Opcode and an Instruction
/// Format: U-Type.
/// Action: rd = imm
pub struct Lui {
    ty: TypeU,
}

impl Opcode for Lui {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeU::from(data),
        })
    }
}

impl Instruction for Lui {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Load the immediate value into the register.
        let reg = engine.register_mut(self.ty.rd)?;
        *reg = self.ty.imm;

        // Go to next instruction
        let pc = engine.pc_mut();
        *pc += INSTRUCTION_SIZE;

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lui() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let lui = Lui {
            ty: TypeU { rd: 1, imm: 0x1000 },
        };

        let result = lui.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x1000);
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }
}
