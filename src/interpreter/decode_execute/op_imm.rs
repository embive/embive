use crate::instruction::embive::OpImm;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for OpImm {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        let rs1 = interpreter.registers.cpu.get(inst.rs1)?;
        let imm = inst.imm;

        if inst.rd_rs2 != 0 {
            // rd = 0 means its a HINT instruction, just ignore it.
            let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
            *rd = match inst.funct3 {
                Self::ADDI_FUNC3 => rs1.wrapping_add(imm),
                Self::SLLI_FUNC3 => rs1.wrapping_shl(imm as u32 & 0b11111),
                Self::SLTI_FUNC3 => (rs1 < imm) as u8 as i32,
                Self::SLTIU_FUNC3 => ((rs1 as u32) < (imm as u32)) as u8 as i32,
                Self::XORI_FUNC3 => rs1 ^ imm,
                Self::SRLI_SRAI_FUNC3 => {
                    if (imm & (0b1 << 10)) != 0 {
                        // Sra (Arithmetic shift right, fill with sign bit)
                        rs1.wrapping_shr(imm as u32 & 0b11111)
                    } else {
                        // Srl (Logical shift right, fill with zero)
                        (rs1 as u32).wrapping_shr(imm as u32 & 0b11111) as i32
                    }
                }
                Self::ORI_FUNC3 => rs1 | imm,
                Self::ANDI_FUNC3 => rs1 & imm,
                _ => return Err(Error::InvalidInstruction(data)),
            };
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        // Continue execution
        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeI},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_addi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let addi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            funct3: OpImm::ADDI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode_execute(addi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x101);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_addi_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let addi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -100,
            funct3: OpImm::ADDI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode_execute(addi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -99);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_xori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let xori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            funct3: OpImm::XORI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x123;

        let result = OpImm::decode_execute(xori.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x023);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_xori_not() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let xori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -1,
            funct3: OpImm::XORI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(xori.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), !0x1234);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_ori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let ori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            funct3: OpImm::ORI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(ori.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 | 0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_ori_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let ori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x100,
            funct3: OpImm::ORI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(ori.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 | -0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_andi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let andi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            funct3: OpImm::ANDI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(andi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 & 0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_slli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let slli = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101,
            funct3: OpImm::SLLI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(slli.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 << 0b101
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_srli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let srli = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101,
            funct3: OpImm::SRLI_SRAI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(srli.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            ((-0x1234i32 as u32) >> 0b101) as i32
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_srai() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let srai = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101 | (0b1 << 10),
            funct3: OpImm::SRLI_SRAI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(srai.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            -0x1234 >> 0b101
        );
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_slti_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x123,
            funct3: OpImm::SLTI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode_execute(slti.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_slti_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: OpImm::SLTI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(slti.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_slti_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: OpImm::SLTI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode_execute(slti.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_slti_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x1000,
            funct3: OpImm::SLTI_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(slti.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_sltiu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x123,
            funct3: OpImm::SLTIU_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode_execute(sltiu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_sltiu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: OpImm::SLTIU_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(sltiu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_sltiu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: OpImm::SLTIU_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode_execute(sltiu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }

    #[test]
    fn test_sltiu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x100,
            funct3: OpImm::SLTIU_FUNC3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(sltiu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::SIZE as u32);
    }
}
