use crate::instruction::embive::CBnez;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CBnez {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Branch if rs1 is not zero
        if interpreter.registers.cpu.get(inst.rs1)? != 0 {
            interpreter.program_counter = interpreter.program_counter.wrapping_add_signed(inst.imm);
        } else {
            // Go to next instruction
            interpreter.program_counter =
                interpreter.program_counter.wrapping_add(Self::SIZE as u32);
        }

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCB4},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cneqz() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let bnez = TypeCB4 { imm: 0x4, rs1: 8 };

        *interpreter.registers.cpu.get_mut(8).unwrap() = 0x1;

        let result = CBnez::decode_execute(bnez.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x4);
    }

    #[test]
    fn test_cneqz_zero() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let bnez = TypeCB4 { imm: 0x4, rs1: 8 };

        let result = CBnez::decode_execute(bnez.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
