use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeJ;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

/// Jump And Link
/// Both an Opcode and an Instruction
/// Format: J-Type.
/// Action: rd = PC+4; PC += imm
pub struct Jal {
    ty: TypeJ,
}

impl Opcode for Jal {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeJ::from(data),
        })
    }
}

impl Instruction for Jal {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Get the current program counter.
        let pc = *engine.pc_mut();

        // Load pc + instruction size into the destination register.
        if self.ty.rd != 0 {
            let reg = engine.register_mut(self.ty.rd)?;
            *reg = pc + INSTRUCTION_SIZE;
        }

        // Set the program counter to the new address.
        let pc = engine.pc_mut();
        *pc += self.ty.imm as i32;

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let jal = Jal {
            ty: TypeJ { rd: 1, imm: 0x1000 },
        };

        let result = jal.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x5);
        assert_eq!(*engine.pc_mut(), 0x1 + 0x1000);
    }
}
