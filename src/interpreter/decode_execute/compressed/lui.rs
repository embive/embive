use crate::instruction::embive::CLui;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CLui {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Load the upper immediate value into the register.
        if self.0.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(self.0.rd_rs1)?;
            *rs1 = self.0.imm;
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
        format::{Format, TypeCI3},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_clui() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lui = TypeCI3 {
            rd_rs1: 1,
            imm: 21 << 12,
        };

        let result = CLui::decode(lui.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 21 << 12);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
