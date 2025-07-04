use crate::instruction::embive::Branch;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for Branch {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;
        let rs2 = interpreter.registers.cpu.get(self.0.rs2)?;

        let branch = match self.0.func {
            Self::BEQ_FUNC => rs1 == rs2,
            Self::BNE_FUNC => rs1 != rs2,
            Self::BLT_FUNC => rs1 < rs2,
            Self::BGE_FUNC => rs1 >= rs2,
            Self::BLTU_FUNC => (rs1 as u32) < (rs2 as u32),
            Self::BGEU_FUNC => (rs1 as u32) >= (rs2 as u32),
            _ => return Err(Error::InvalidInstruction(interpreter.program_counter)),
        };

        interpreter.program_counter = if branch {
            // Branch to new address
            interpreter.program_counter.wrapping_add_signed(self.0.imm)
        } else {
            // Go to next instruction
            interpreter
                .program_counter
                .wrapping_add(Self::size() as u32)
        };

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeB},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_beq_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: -0x100,
            func: Branch::BEQ_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1u32.wrapping_sub(0x100u32));
    }

    #[test]
    fn test_beq_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BEQ_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_beq_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BEQ_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }

    #[test]
    fn test_bne_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BNE_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }

    #[test]
    fn test_bne_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BNE_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_blt_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BLT_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_blt_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BLT_FUNC,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }

    #[test]
    fn test_bge_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGE_FUNC,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bge_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGE_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bge_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGE_FUNC,
            rs1: 1,
            rs2: 2,
        };
        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }

    #[test]
    fn test_bltu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BLTU_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bltu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BLTU_FUNC,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -0x2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }

    #[test]
    fn test_bgeu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGEU_FUNC,
            rs1: 2,
            rs2: 1,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGEU_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            func: Branch::BGEU_FUNC,
            rs1: 1,
            rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;
        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode(branch.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1 + Branch::size() as u32);
    }
}
