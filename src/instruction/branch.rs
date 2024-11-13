use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeB;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

const BEQ_FUNCT3: u8 = 0b000;
const BNE_FUNCT3: u8 = 0b001;
const BLT_FUNCT3: u8 = 0b100;
const BGE_FUNCT3: u8 = 0b101;
const BLTU_FUNCT3: u8 = 0b110;
const BGEU_FUNCT3: u8 = 0b111;

/// Branch OpCode
/// Instructions: Beq, Bne, Blt, Bqe, Bltu, Bgeu
/// Format: B-Type.
pub struct Branch {
    ty: TypeB,
}

impl Opcode for Branch {
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError> {
        Ok(Self {
            ty: TypeB::from(data),
        })
    }
}

impl Instruction for Branch {
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        let rs1 = engine.register(self.ty.rs1)?;
        let rs2 = engine.register(self.ty.rs2)?;
        let branch;

        match self.ty.funct3 {
            BEQ_FUNCT3 => {
                branch = rs1 == rs2;
            }
            BNE_FUNCT3 => {
                branch = rs1 != rs2;
            }
            BLT_FUNCT3 => {
                branch = rs1 < rs2;
            }
            BGE_FUNCT3 => {
                branch = rs1 >= rs2;
            }
            BLTU_FUNCT3 => {
                branch = (rs1 as u32) < (rs2 as u32);
            }
            BGEU_FUNCT3 => {
                branch = (rs1 as u32) >= (rs2 as u32);
            }
            _ => {
                return Err(EmbiveError::InvalidInstruction);
            }
        }

        let pc = engine.pc_mut();
        if branch {
            // Branch to new address
            *pc += self.ty.imm as i32;
        } else {
            // Go to next instruction
            *pc += INSTRUCTION_SIZE;
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beq_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BEQ_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = -0x1;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_beq_not_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BEQ_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x2;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bne_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BNE_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bne_not_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BNE_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = -0x2;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_blt_less_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BLT_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x2;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_blt_greater_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BLT_FUNCT3,
                rs1: 2,
                rs2: 1,
            },
        };

        *engine.register_mut(1).unwrap() = -0x2;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bge_greater_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGE_FUNCT3,
                rs1: 2,
                rs2: 1,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x2;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_bge_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGE_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_bge_less_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGE_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = -0x2;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bltu_less_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BLTU_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x2;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_bltu_greater_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BLTU_FUNCT3,
                rs1: 2,
                rs2: 1,
            },
        };

        *engine.register_mut(1).unwrap() = -0x2;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bgeu_greater_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGEU_FUNCT3,
                rs1: 2,
                rs2: 1,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_bgeu_equal() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGEU_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = 0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1001);
    }

    #[test]
    fn test_bgeu_less_than() {
        let mut engine = Engine::new(&[], &mut []).unwrap();
        *engine.pc_mut() = 0x1;
        let branch = Branch {
            ty: TypeB {
                imm: 0x1000,
                funct3: BGEU_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.register_mut(1).unwrap() = 0x1;
        *engine.register_mut(2).unwrap() = -0x1;

        let result = branch.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.pc_mut(), 0x1 + INSTRUCTION_SIZE);
    }
}
