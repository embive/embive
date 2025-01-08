use crate::instruction::embive::Jal;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for Jal {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Load pc + instruction size into the destination register.
        if inst.rd != 0 {
            let reg = interpreter.registers.cpu.get_mut(inst.rd)?;
            *reg = interpreter.program_counter.wrapping_add(Self::SIZE as u32) as i32;
        }

        // Set the program counter to the new address.
        interpreter.program_counter = interpreter.program_counter.wrapping_add_signed(inst.imm);

        // Continue execution
        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeJ},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_jal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter.program_counter = 0x1;
        let jal = TypeJ { rd: 1, imm: 0x1000 };

        let result = Jal::decode_execute(jal.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(interpreter.program_counter, 0x1 + 0x1000);
    }
}
