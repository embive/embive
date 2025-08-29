use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::Lui;
use crate::interpreter::utils::likely;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for Lui {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        if likely(self.0.rd != 0) {
            // rd = 0 means its a HINT instruction, just ignore it.
            // Load the immediate value into the register.
            let reg = interpreter.registers.cpu.get_mut(self.0.rd)?;
            *reg = self.0.imm;
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
        format::{Format, TypeU},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_lui() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.program_counter = 0x1;
        let lui = TypeU { rd: 1, imm: 0x1000 };

        let result = Lui::decode(lui.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x1000);
        assert_eq!(interpreter.program_counter, 0x1 + Lui::size() as u32);
    }
}
