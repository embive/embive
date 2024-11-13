use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

/// Jump And Link Reg
/// Both an Opcode and an Instruction
/// Format: I-Type.
/// Action: rd = PC+4; PC = rs1 + imm
pub struct Jalr {
    ty: TypeI,
}

impl Opcode for Jalr {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeI::from(data),
        })
    }
}

impl Instruction for Jalr {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Get the current program counter.
        let pc = *engine.pc_mut();

        // Get the value of the source register.
        let rs1 = engine.register(self.ty.rs1)?;

        // Load pc + instruction size into the destination register (if not unconditional).
        if self.ty.rd != 0 {
            let rd = engine.register_mut(self.ty.rd)?;
            *rd = pc + INSTRUCTION_SIZE;
        }

        // Set the program counter to the new address.
        let pc = engine.pc_mut();
        *pc = rs1 + self.ty.imm as i32;

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jlr() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let jalr = Jalr {
            ty: TypeI { funct3: 0x0, rd: 1, rs1: 2, imm: 0x1000 },
        };

        *engine.register_mut(2).unwrap() = 0x2000;

        let result = jalr.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x5);
        assert_eq!(*engine.pc_mut(), 0x3000);
    }

    #[test]
    fn test_jlr_same_reg() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let jalr = Jalr {
            ty: TypeI { funct3: 0x0, rd: 1, rs1: 1, imm: 0x1000 },
        };

        *engine.register_mut(1).unwrap() = 0x2000;

        let result = jalr.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x5);
        assert_eq!(*engine.pc_mut(), 0x3000);
    }
}
