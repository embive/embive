use crate::instruction::embive::CJrMv;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CJrMv {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        if inst.rs2 == 0 {
            // JR (Jump Register)
            let rd_rs1 = interpreter.registers.cpu.get(inst.rd_rs1)?;

            interpreter.program_counter = rd_rs1 as u32;
        } else {
            // MV (Move)
            let rs2 = interpreter.registers.cpu.get(inst.rs2)?;
            let rd_rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;

            *rd_rs1 = rs2;

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
        format::{Format, TypeCR},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cjr() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let jr = TypeCR { rd_rs1: 1, rs2: 0 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 4;

        let result = CJrMv::decode_execute(jr.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x4);
    }

    #[test]
    fn test_cmv() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let mv = TypeCR { rd_rs1: 1, rs2: 2 };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 4;

        let result = CJrMv::decode_execute(mv.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 4);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
