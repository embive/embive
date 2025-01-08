use crate::instruction::embive::CXor;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CXor {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Xor operation
        if inst.rd_rs1 != 0 {
            let rs2 = interpreter.registers.cpu.get(inst.rs2)?;
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;

            *rs1 ^= rs2;
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCS},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cxor() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let xor = TypeCS { rd_rs1: 1, rs2: 2 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 5;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;

        let result = CXor::decode_execute(xor.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 6);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
