use crate::instruction::embive::Branch;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for Branch {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        let rs1 = interpreter.registers.cpu.get(inst.rs1)?;
        let rs2 = interpreter.registers.cpu.get(inst.rs2)?;

        let branch = match inst.funct3 {
            Self::BEQ_FUNCT3 => rs1 == rs2,
            Self::BNE_FUNCT3 => rs1 != rs2,
            Self::BLT_FUNCT3 => rs1 < rs2,
            Self::BGE_FUNCT3 => rs1 >= rs2,
            Self::BLTU_FUNCT3 => (rs1 as u32) < (rs2 as u32),
            Self::BGEU_FUNCT3 => (rs1 as u32) >= (rs2 as u32),
            _ => return Err(Error::InvalidInstruction(data)),
        };

        interpreter.program_counter = if branch {
            // Branch to new address
            interpreter.program_counter.wrapping_add_signed(inst.imm)
        } else {
            // Go to next instruction
            interpreter.program_counter.wrapping_add(Self::SIZE as u32)
        };

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeB},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_beq_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: -0x100,
            funct3: Branch::BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1u32.wrapping_sub(0x100u32));
    }

    #[test]
    fn test_beq_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_beq_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }

    #[test]
    fn test_bne_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BNE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }

    #[test]
    fn test_bne_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BNE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_blt_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BLT_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_blt_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BLT_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }

    #[test]
    fn test_bge_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGE_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bge_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bge_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };
        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }

    #[test]
    fn test_bltu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BLTU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bltu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BLTU_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }

    #[test]
    fn test_bgeu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGEU_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGEU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: Branch::BGEU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::SIZE as u32);
    }
}
