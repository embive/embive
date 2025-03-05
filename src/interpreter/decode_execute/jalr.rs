use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::Jalr;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for Jalr {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Get the value of the source register.
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;

        // Load pc + instruction size into the destination register (if not unconditional).
        if self.0.rd_rs2 != 0 {
            let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
            *rd = interpreter
                .program_counter
                .wrapping_add(Self::size() as u32) as i32;
        }

        // Set the program counter to the new address.
        interpreter.program_counter = (rs1 as u32).wrapping_add_signed(self.0.imm);

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
    fn test_jlr_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        interpreter.program_counter = 0x1;
        let jalr = TypeI {
            func: 0x0,
            rd_rs2: 1,
            rs1: 2,
            imm: -0x100,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = -0x200;

        let result = Jalr::decode(jalr.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, (-0x200i32 + -0x100i32) as u32);
    }

    #[test]
    fn test_jlr() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        interpreter.program_counter = 0x1;
        let jalr = TypeI {
            func: 0x0,
            rd_rs2: 1,
            rs1: 2,
            imm: 0x100,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x200;

        let result = Jalr::decode(jalr.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, 0x300);
    }

    #[test]
    fn test_jlr_same_reg() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        interpreter.program_counter = 0x1;
        let jalr = TypeI {
            func: 0x0,
            rd_rs2: 1,
            rs1: 1,
            imm: 0x100,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x200;

        let result = Jalr::decode(jalr.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, 0x300);
    }
}
