use crate::instruction::embive::CJal;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CJal {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Load pc + instruction size into the return address register.
        let ra = interpreter.registers.cpu.get_mut(CPURegister::RA as u8)?;
        *ra = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32) as i32;

        // Set the program counter to the new address.
        interpreter.program_counter = interpreter.program_counter.wrapping_add_signed(self.0.imm);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCJ},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cjal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let jal = TypeCJ { imm: 0xc };

        let result = CJal::decode(jal.to_embive()).execute(&mut interpreter);
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
