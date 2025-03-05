use crate::instruction::embive::CBeqz;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CBeqz {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Branch if rs1 is zero
        if interpreter.registers.cpu.get(self.0.rs1)? == 0 {
            interpreter.program_counter =
                interpreter.program_counter.wrapping_add_signed(self.0.imm);
        } else {
            // Go to next instruction
            interpreter.program_counter = interpreter
                .program_counter
                .wrapping_add(Self::size() as u32);
        }

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCB4},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cbeqz() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let beqz = TypeCB4 { imm: 0x4, rs1: 8 };

        *interpreter.registers.cpu.get_mut(8).unwrap() = 0x1;

        let result = CBeqz::decode(beqz.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x2);
    }

    #[test]
    fn test_cbeqz_zero() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let beqz = TypeCB4 { imm: 0x4, rs1: 8 };

        let result = CBeqz::decode(beqz.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x4);
    }
}
