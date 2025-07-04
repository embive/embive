use crate::instruction::embive::CSlli;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CSlli {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Left shift
        if self.0.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(self.0.rd_rs1)?;
            *rs1 = rs1.wrapping_shl(self.0.imm as u32);
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
        format::{Format, TypeCI4},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cslli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let srli = TypeCI4 { rd_rs1: 1, imm: 3 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 10;

        let result = CSlli::decode(srli.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 10 << 3);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
