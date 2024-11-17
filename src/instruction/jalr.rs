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
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction {
        Self {
            ty: TypeI::from(data),
        }
    }
}

impl Instruction for Jalr {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        // Get the value of the source register.
        let rs1 = engine.registers.get(self.ty.rs1)?;

        // Load pc + instruction size into the destination register (if not unconditional).
        if self.ty.rd != 0 {
            let rd = engine.registers.get_mut(self.ty.rd)?;
            *rd = engine.pc.wrapping_add(INSTRUCTION_SIZE) as i32;
        }

        // Set the program counter to the new address.
        engine.pc = (rs1 as u32).wrapping_add_signed(self.ty.imm as i32);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jlr_negative() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.pc = 0x1;
        let jalr = Jalr {
            ty: TypeI {
                funct3: 0x0,
                rd: 1,
                rs1: 2,
                imm: -0x1000,
            },
        };

        *engine.registers.get_mut(2).unwrap() = -0x2000;

        let result = jalr.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.pc, (-0x2000i32 + -0x1000i32) as u32);
    }

    #[test]
    fn test_jlr() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.pc = 0x1;
        let jalr = Jalr {
            ty: TypeI {
                funct3: 0x0,
                rd: 1,
                rs1: 2,
                imm: 0x1000,
            },
        };

        *engine.registers.get_mut(2).unwrap() = 0x2000;

        let result = jalr.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.pc, 0x3000);
    }

    #[test]
    fn test_jlr_same_reg() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.pc = 0x1;
        let jalr = Jalr {
            ty: TypeI {
                funct3: 0x0,
                rd: 1,
                rs1: 1,
                imm: 0x1000,
            },
        };

        *engine.registers.get_mut(1).unwrap() = 0x2000;

        let result = jalr.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.pc, 0x3000);
    }
}
