use crate::instruction::embive::CAndi;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CAndi {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // And operation
        if inst.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;
            *rs1 &= inst.imm;
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCB2},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_candi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let andi = TypeCB2 {
            rd_rs1: 1,
            imm: 0x5,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 1;

        let result = CAndi::decode_execute(andi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
