use crate::instruction::embive::CLi;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CLi {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Load the immediate value into the register.
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
        format::{Format, TypeCI1},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let li = TypeCI1 {
            rd_rs1: 1,
            imm: 0x4,
        };

        let result = CLi::decode(li.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x4);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
