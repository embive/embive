use crate::instruction::embive::CJ;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CJ {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Set the program counter to the new address.
        interpreter.program_counter = interpreter.program_counter.wrapping_add_signed(inst.imm);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCJ},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cj() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let jal = TypeCJ { imm: 0xc };

        let result = CJ::decode_execute(jal.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0xc);
    }
}
