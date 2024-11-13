use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

const LB_FUNCT3: u8 = 0b000;
const LH_FUNCT3: u8 = 0b001;
const LW_FUNCT3: u8 = 0b010;
const LBU_FUNCT3: u8 = 0b100;
const LHU_FUNCT3: u8 = 0b101;

/// Load OpCode
/// Instructions: Lb, Lh, Lw, Lbu, Lhu
/// Format: I-Type.
pub struct Load {
    ty: TypeI,
}

impl Opcode for Load {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeI::from(data),
        })
    }
}

impl Instruction for Load {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        let rs1 = engine.register(self.ty.rs1)?;
        let result;

        match self.ty.funct3 {
            LB_FUNCT3 => {
                let address = rs1 + self.ty.imm as i32;
                let data = i8::from_le_bytes(engine.load(address)?);
                result = data as i32;
            }
            LH_FUNCT3 => {
                let address = rs1 + self.ty.imm as i32;
                let data = i16::from_le_bytes(engine.load(address)?);
                result = data as i32;
            }
            LW_FUNCT3 => {
                let address = rs1 + self.ty.imm as i32;
                let data = i32::from_le_bytes(engine.load(address)?);
                result = data;
            }
            LBU_FUNCT3 => {
                let address = rs1 + self.ty.imm as i32;
                let data = u8::from_le_bytes(engine.load(address)?);
                result = data as i32;
            }
            LHU_FUNCT3 => {
                let address = rs1 + self.ty.imm as i32;
                let data = u16::from_le_bytes(engine.load(address)?);
                result = data as i32;
            }
            _ => {
                return Err(EmbiveError::InvalidInstruction);
            }
        }

        // Store the result in the destination register
        let rd = engine.register_mut(self.ty.rd)?;
        *rd = result;

        // Go to next instruction
        let pc = engine.pc_mut();
        *pc += INSTRUCTION_SIZE;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MEMORY_OFFSET;

    #[test]
    fn test_lb() {
        let mut memory = [0x0; 2];
        memory[1] = 0x12;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lb = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LB_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lb.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x12);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lb_negative() {
        let mut memory = [0x0; 2];
        memory[1] = -0x12i8 as u8;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lb = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LB_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lb.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), -0x12);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lh() {
        let mut memory = [0x0; 3];
        memory[1] = 0x12;
        memory[2] = 0x34;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lh = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LH_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lh.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x3412);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lh_negative() {
        let mut memory = (-28098i16).to_le_bytes();

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lh = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LH_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lh.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), -28098);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lw() {
        let mut memory = [0x0; 5];
        memory[1] = 0x12;
        memory[2] = 0x34;
        memory[3] = 0x56;
        memory[4] = 0x78;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lw = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LW_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lw.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x78563412);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lw_negative() {
        let mut memory = (-19088744i32).to_le_bytes();

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lw = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LW_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lw.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), -19088744);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lbu() {
        let mut memory = [0x0; 2];
        memory[1] = 0x12;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lbu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LBU_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lbu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x12);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lbu_negative() {
        let mut memory = [0x0; 2];
        memory[1] = -0x12i8 as u8;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lbu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LBU_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lbu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), (-0x12i8 as u8) as i32);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lhu() {
        let mut memory = [0x0; 3];
        memory[1] = 0x12;
        memory[2] = 0x34;

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lhu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LHU_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lhu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0x3412);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lhu_negative() {
        let mut memory = (-28098i16).to_le_bytes();

        let mut engine = Engine::new(&[], &mut memory).unwrap();
        let lhu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LHU_FUNCT3,
            },
        };
        *engine.register_mut(2).unwrap() = MEMORY_OFFSET;

        let result = lhu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), (-28098i16 as u16) as i32);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }
}
