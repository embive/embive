use crate::instruction::embive::CAddi;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CAddi {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Add Immediate
        if inst.rd_rs1 != 0 {
            let rs1 = interpreter.registers.cpu.get_mut(inst.rd_rs1)?;
            *rs1 = rs1.wrapping_add(inst.imm);
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCI1},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_caddi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let addi = TypeCI1 {
            rd_rs1: 1,
            imm: 0x4,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1;

        let result = CAddi::decode_execute(addi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
