use crate::instruction::embive::Auipc;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for Auipc {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // rd = 0 means its a HINT instruction, just ignore it.
        if inst.rd != 0 {
            // Load the immediate value + pc into the register.
            let reg = interpreter.registers.cpu.get_mut(inst.rd)?;
            *reg = interpreter.program_counter.wrapping_add_signed(inst.imm) as i32;
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        // Continue execution
        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeU},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_auipc() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let auipc = TypeU { rd: 1, imm: 0x1000 };

        let result = Auipc::decode_execute(auipc.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x1001);
        assert_eq!(interpreter.program_counter, 0x1 + Auipc::SIZE as u32);
    }

    #[test]
    fn test_auipc_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let auipc = TypeU {
            rd: 1,
            imm: -0x1000,
        };

        let result = Auipc::decode_execute(auipc.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -0xfff);
        assert_eq!(interpreter.program_counter, 0x1 + Auipc::SIZE as u32);
    }
}
