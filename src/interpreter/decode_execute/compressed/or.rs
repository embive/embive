use crate::instruction::embive::COr;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for COr {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Or operation
        if inst.rd_rs1 != 0 {
            let rs2 = interpreter.registers.cpu.get(inst.rs2)?;
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;

            *rs1 |= rs2;
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
    fn test_cor() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let or = TypeCS { rd_rs1: 1, rs2: 2 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 1;

        let result = COr::decode_execute(or.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 3);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
