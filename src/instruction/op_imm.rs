use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};
use crate::memory::Memory;

const ADDI_FUNC3: u8 = 0b000;
const XORI_FUNC3: u8 = 0b100;
const ORI_FUNC3: u8 = 0b110;
const ANDI_FUNC3: u8 = 0b111;
const SLLI_FUNC3: u8 = 0b001;
const SRLI_SRAI_FUNC3: u8 = 0b101;
const SLTI_FUNC3: u8 = 0b010;
const SLTIU_FUNC3: u8 = 0b011;

/// Operation Immediate OpCode
/// Instructions: Addi, Xori, Ori, Andi, Slli, Srli, Srai, Slti, Sltiu
/// Format: I-Type.
pub struct OpImm {
    ty: TypeI,
}

impl<M: Memory> Opcode<M> for OpImm {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction<M> {
        Self {
            ty: TypeI::from(data),
        }
    }
}

impl<M: Memory> Instruction<M> for OpImm {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let rs1 = engine.registers.get(self.ty.rs1)?;
        let imm = self.ty.imm as i32;

        if self.ty.rd != 0 {
            // rd = 0 means its a HINT instruction, just ignore it.
            let rd = engine.registers.get_mut(self.ty.rd)?;
            *rd = match self.ty.funct3 {
                ADDI_FUNC3 => rs1.wrapping_add(imm),
                SLLI_FUNC3 => rs1 << (imm & 0b11111),
                SLTI_FUNC3 => (rs1 < imm) as u8 as i32,
                SLTIU_FUNC3 => ((rs1 as u32) < (imm as u32)) as u8 as i32,
                XORI_FUNC3 => rs1 ^ imm,
                SRLI_SRAI_FUNC3 => {
                    if (imm & (0b1 << 10)) != 0 {
                        // Sra (Arithmetic shift right, fill with sign bit)
                        rs1 >> (imm & 0b11111)
                    } else {
                        // Srl (Logical shift right, fill with zero)
                        ((rs1 as u32) >> ((imm & 0b11111) as u32)) as i32
                    }
                }
                ORI_FUNC3 => rs1 | imm,
                ANDI_FUNC3 => rs1 & imm,
                _ => return Err(EmbiveError::InvalidInstruction),
            };
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_addi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let addi = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: ADDI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 1;

        let result = addi.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x1001);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_addi_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let addi = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: -1000,
                funct3: ADDI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 1;

        let result = addi.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -999);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let xori = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: XORI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = xori.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x0234);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xori_not() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let xori = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: -1,
                funct3: XORI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = xori.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), !0x1234);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let ori = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: ORI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = ori.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x1234 | 0x1000);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ori_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let ori = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: -0x1000,
                funct3: ORI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = ori.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x1234 | -0x1000);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_andi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let andi = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: ANDI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = andi.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x1234 & 0x1000);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slli = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0b101,
                funct3: SLLI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = slli.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x1234 << 0b101);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let srli = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0b101,
                funct3: SRLI_SRAI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = -0x1234;

        let result = srli.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            ((-0x1234i32 as u32) >> 0b101) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srai() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let srai = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0b101 | (0b1 << 10),
                funct3: SRLI_SRAI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = -0x1234;

        let result = srai.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -0x1234 >> 0b101);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1234,
                funct3: SLTI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1000;

        let result = slti.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: SLTI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = slti.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: SLTI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1000;

        let result = slti.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: -0x1000,
                funct3: SLTI_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = -0x1234;

        let result = slti.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1234,
                funct3: SLTIU_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1000;

        let result = sltiu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: SLTIU_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = sltiu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1000,
                funct3: SLTIU_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = 0x1000;

        let result = sltiu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = OpImm {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: -0x1000,
                funct3: SLTIU_FUNC3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = -0x1234;

        let result = sltiu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }
}
