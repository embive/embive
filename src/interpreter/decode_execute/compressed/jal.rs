use crate::instruction::embive::CJal;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CJal {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Load pc + instruction size into the return address register.
        let ra = interpreter.registers.cpu.get_mut(CPURegister::RA as u8)?;
        *ra = interpreter.program_counter.wrapping_add(Self::SIZE as u32) as i32;

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
    fn test_cjal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let jal = TypeCJ { imm: 0xc };

        let result = CJal::decode_execute(jal.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::RA as u8)
                .unwrap(),
            0x2
        );
        assert_eq!(interpreter.program_counter, 0xc);
    }
}
