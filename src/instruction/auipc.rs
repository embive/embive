use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeU;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

/// Add Upper Immediate to Program Counter
/// Both an Opcode and an Instruction
/// Format: U-Type.
/// Action: rd = PC + imm
pub struct Auipc {
    ty: TypeU,
}

impl Opcode for Auipc {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeU::from(data),
        })
    }
}

impl Instruction for Auipc {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Get the current program counter.
        let pc = *engine.pc_mut();

        // Load the immediate value into the register.
        let reg = engine.register_mut(self.ty.rd)?;
        *reg = pc + self.ty.imm;

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
    fn test_auipc() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let auipc = Auipc {
            ty: TypeU { rd: 1, imm: 0x1000 },
        };

        let result = auipc.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x1001);
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }
}
