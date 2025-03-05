use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::Jal;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for Jal {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Load pc + instruction size into the destination register.
        if self.0.rd != 0 {
            let reg = interpreter.registers.cpu.get_mut(self.0.rd)?;
            *reg = interpreter
                .program_counter
                .wrapping_add(Self::size() as u32) as i32;
        }

        // Set the program counter to the new address.
        interpreter.program_counter = interpreter.program_counter.wrapping_add_signed(self.0.imm);

        // Continue execution
        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeJ},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_jal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        interpreter.program_counter = 0x1;
        let jal = TypeJ { rd: 1, imm: 0x1000 };

        let result = Jal::decode(jal.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, 0x1 + 0x1000);
    }
}
