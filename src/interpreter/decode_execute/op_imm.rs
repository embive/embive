use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::OpImm;
use crate::interpreter::utils::likely;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for OpImm {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;
        let imm = self.0.imm;

        if likely(self.0.rd_rs2 != 0) {
            // rd = 0 means its a HINT instruction, just ignore it.
            let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
            *rd = match self.0.func {
                Self::ADDI_FUNC => rs1.wrapping_add(imm),
                Self::SLLI_FUNC => rs1.wrapping_shl(imm as u32 & 0b11111),
                Self::SLTI_FUNC => (rs1 < imm) as u8 as i32,
                Self::SLTIU_FUNC => ((rs1 as u32) < (imm as u32)) as u8 as i32,
                Self::XORI_FUNC => rs1 ^ imm,
                Self::SRLI_SRAI_FUNC => {
                    if (imm & (0b1 << 10)) != 0 {
                        // Sra (Arithmetic shift right, fill with sign bit)
                        rs1.wrapping_shr(imm as u32 & 0b11111)
                    } else {
                        // Srl (Logical shift right, fill with zero)
                        (rs1 as u32).wrapping_shr(imm as u32 & 0b11111) as i32
                    }
                }
                Self::ORI_FUNC => rs1 | imm,
                Self::ANDI_FUNC => rs1 & imm,
                _ => return Err(Error::InvalidInstruction(interpreter.program_counter)),
            };
        }

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        // Continue execution
        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeI},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_addi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let addi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            func: OpImm::ADDI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode(addi.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x101);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_addi_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let addi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -100,
            func: OpImm::ADDI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode(addi.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -99);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_xori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let xori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            func: OpImm::XORI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x123;

        let result = OpImm::decode(xori.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x023);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_xori_not() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let xori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -1,
            func: OpImm::XORI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(xori.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), !0x1234);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_ori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let ori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            func: OpImm::ORI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(ori.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 | 0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_ori_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let ori = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x100,
            func: OpImm::ORI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(ori.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 | -0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_andi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let andi = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
            func: OpImm::ANDI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(andi.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 & 0x100
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_slli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let slli = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101,
            func: OpImm::SLLI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(slli.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0x1234 << 0b101
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_srli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let srli = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101,
            func: OpImm::SRLI_SRAI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode(srli.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            ((-0x1234i32 as u32) >> 0b101) as i32
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_srai() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let srai = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0b101 | (0b1 << 10),
            func: OpImm::SRLI_SRAI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode(srai.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            -0x1234 >> 0b101
        );
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_slti_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x123,
            func: OpImm::SLTI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode(slti.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_slti_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            func: OpImm::SLTI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(slti.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_slti_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            func: OpImm::SLTI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode(slti.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_slti_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let slti = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x1000,
            func: OpImm::SLTI_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode(slti.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_sltiu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x123,
            func: OpImm::SLTIU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode(sltiu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_sltiu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            func: OpImm::SLTIU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode(sltiu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_sltiu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1000,
            func: OpImm::SLTIU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode(sltiu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }

    #[test]
    fn test_sltiu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let sltiu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: -0x100,
            func: OpImm::SLTIU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode(sltiu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpImm::size() as u32);
    }
}
