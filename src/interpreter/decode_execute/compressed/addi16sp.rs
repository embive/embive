use crate::instruction::embive::CAddi16sp;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CAddi16sp {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Add Immediate to SP
        let sp = interpreter.registers.cpu.get_mut(CPURegister::SP as u8)?;
        *sp = sp.wrapping_add(self.0.imm);

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
        format::{Format, TypeCI2},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_caddi16spn() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        *interpreter
            .registers
            .cpu
            .get_mut(CPURegister::SP as u8)
            .unwrap() = 0x1;

        let addi16sp = TypeCI2 { imm: 96, rd_rs1: 2 };

        let result = CAddi16sp::decode(addi16sp.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::SP as u8)
                .unwrap(),
            97
        );
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
