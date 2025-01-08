use crate::instruction::embive::CSlli;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CSlli {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Left shift
        if inst.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;
            *rs1 = rs1.wrapping_shl(inst.imm as u32);
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCI4},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cslli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let srli = TypeCI4 { rd_rs1: 1, imm: 3 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 10;

        let result = CSlli::decode_execute(srli.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 10 << 3);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
