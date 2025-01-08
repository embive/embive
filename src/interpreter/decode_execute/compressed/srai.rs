use crate::instruction::embive::CSrai;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CSrai {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Arithmetic right shift
        if inst.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;
            *rs1 = rs1.wrapping_shr(inst.imm as u32);
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCB1},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_csrai() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let srai = TypeCB1 { rd_rs1: 1, imm: 3 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = -1;

        let result = CSrai::decode_execute(srai.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -1i32 >> 3);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
