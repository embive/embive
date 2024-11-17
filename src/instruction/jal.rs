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
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction {
        Self {
            ty: TypeJ::from(data),
        }
    }
}

impl Instruction for Jal {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Load pc + instruction size into the destination register.
        if self.ty.rd != 0 {
            let reg = engine.registers.get_mut(self.ty.rd)?;
            *reg = engine.pc.wrapping_add(INSTRUCTION_SIZE) as i32;
        }

        // Set the program counter to the new address.
        engine.pc = engine.pc.wrapping_add_signed(self.ty.imm);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jal() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.pc = 0x1;
        let jal = Jal {
            ty: TypeJ { rd: 1, imm: 0x1000 },
        };

        let result = jal.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.pc, 0x1 + 0x1000);
    }
}
