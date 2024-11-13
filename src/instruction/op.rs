use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeR;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

const ADD_SUB_FUNCT3: u8 = 0b000;
const SUB_FUNCT7: u8 = 0b0100000;
const XOR_FUNCT3: u8 = 0b100;
const OR_FUNCT3: u8 = 0b110;
const AND_FUNCT3: u8 = 0b111;
const SLL_FUNCT3: u8 = 0b001;
const SRL_SRA_FUNCT3: u8 = 0b101;
const SRA_FUNCT7: u8 = 0b0100000;
const SLT_FUNCT3: u8 = 0b010;
const SLTU_FUNCT3: u8 = 0b011;

/// Operation OpCode
/// Instructions: Add, Sub, Xor, Or, And, Sll, Srl, Sra, Slt, Sltu
/// Format: R-Type.
pub struct Op {
    ty: TypeR,
}

impl Opcode for Op {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeR::from(data),
        })
    }
}

impl Instruction for Op {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        let rs1 = engine.register(self.ty.rs1)?;
        let rs2 = engine.register(self.ty.rs2)?;
        let rd = engine.register_mut(self.ty.rd)?;
        match self.ty.funct3 {
            ADD_SUB_FUNCT3 => {
                if self.ty.funct7 == SUB_FUNCT7 {
                    // Sub (Subtract)
                    *rd = rs1.wrapping_sub(rs2);
                } else {
                    // Add
                    *rd = rs1.wrapping_add(rs2);
                }
            }
            XOR_FUNCT3 => {
                // Xor (Exclusive or)
                *rd = rs1 ^ rs2;
            }
            OR_FUNCT3 => {
                // Or
                *rd = rs1 | rs2;
            }
            AND_FUNCT3 => {
                // And
                *rd = rs1 & rs2;
            }
            SLL_FUNCT3 => {
                // Sll (Logical shift left, fill with zero)
                *rd = rs1 << rs2;
            }
            SRL_SRA_FUNCT3 => {
                if self.ty.funct7 == SRA_FUNCT7 {
                    // Sra (Arithmetic shift right, fill with sign bit)
                    *rd = rs1 >> rs2;
                } else {
                    // Srl (Logical shift right, fill with zero)
                    *rd = ((rs1 as u32) >> (rs2 as u32)) as i32;
                }
            }
            SLT_FUNCT3 => {
                // Slt (Set less than)
                *rd = if rs1 < rs2 { 1 } else { 0 };
            }
            SLTU_FUNCT3 => {
                // Sltu (Set less than, unsigned)
                *rd = if (rs1 as u32) < (rs2 as u32) { 1 } else { 0 };
            }
            _ => return Err(EmbiveError::InvalidInstruction),
        }

        // Go to next instruction
        let pc = engine.pc_mut();
        *pc += INSTRUCTION_SIZE;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: ADD_SUB_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };
        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = 20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 30);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_add_wrapping() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: ADD_SUB_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };
        *engine.register_mut(2).unwrap() = i32::MAX;
        *engine.register_mut(3).unwrap() = 1;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), i32::MIN);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sub() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: ADD_SUB_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: SUB_FUNCT7,
            },
        };
        *engine.register_mut(2).unwrap() = 20;
        *engine.register_mut(3).unwrap() = 10;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 10);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sub_wrapping() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: ADD_SUB_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: SUB_FUNCT7,
            },
        };
        *engine.register_mut(2).unwrap() = i32::MIN;
        *engine.register_mut(3).unwrap() = 1;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), i32::MAX);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xor() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: XOR_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 0b1100;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b0110);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_or() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: OR_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 0b1100;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b1110);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_and() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: AND_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 0b1100;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b1000);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sll() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLL_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 2;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b101000);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srl() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SRL_SRA_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 2;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b10);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srl_negative() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SRL_SRA_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 0xBA987654u32 as i32;
        *engine.register_mut(3).unwrap() = 28;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0xB);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sra() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SRL_SRA_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: SRA_FUNCT7,
            },
        };

        *engine.register_mut(2).unwrap() = 0b1010;
        *engine.register_mut(3).unwrap() = 2;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0b10);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sra_negative() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SRL_SRA_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: SRA_FUNCT7,
            },
        };

        *engine.register_mut(2).unwrap() = 0xBA987654u32 as i32;
        *engine.register_mut(3).unwrap() = 28;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0xFFFFFFFBu32 as i32);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_lower() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLT_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = 20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 1);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_greater() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLT_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 20;
        *engine.register_mut(3).unwrap() = 10;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLT_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 20;
        *engine.register_mut(3).unwrap() = 20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_negative() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLT_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = -20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_lower() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLTU_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = 20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 1);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_greater() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLTU_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 20;
        *engine.register_mut(3).unwrap() = 10;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLTU_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = 10;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 0);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_negative() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        let op = Op {
            ty: TypeR {
                rd: 1,
                funct3: SLTU_FUNCT3,
                rs1: 2,
                rs2: 3,
                funct7: 0,
            },
        };

        *engine.register_mut(2).unwrap() = 10;
        *engine.register_mut(3).unwrap() = -20;

        let result = op.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.register_mut(1).unwrap(), 1);
        assert_eq!(*engine.pc_mut(), INSTRUCTION_SIZE);
    }
}
