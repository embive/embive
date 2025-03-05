use crate::instruction::embive::CSub;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CSub {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Subtract
        if self.0.rd_rs1 != 0 {
            let rs2 = interpreter.registers.cpu.get(self.0.rs2)?;
            let rs1 = interpreter.registers.cpu.get_mut(self.0.rd_rs1)?;

            *rs1 = rs1.wrapping_sub(rs2);
        }

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCS},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_csub() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let sub = TypeCS { rd_rs1: 1, rs2: 2 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = CSub::decode(sub.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
