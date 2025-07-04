use crate::instruction::embive::CEbreakJalrAdd;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CEbreakJalrAdd {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        if self.0.rs2 == 0 {
            if self.0.rd_rs1 == 0 {
                // Ebreak
                // Go to next instruction
                interpreter.program_counter = interpreter
                    .program_counter
                    .wrapping_add(Self::size() as u32);

                // Halt the interpreter
                return Ok(State::Halted);
            } else {
                // Jalr
                let rs1 = interpreter.registers.cpu.get(self.0.rd_rs1)?;

                // Load pc + instruction size into the return address register.
                let ra = interpreter.registers.cpu.get_mut(CPURegister::RA as u8)?;
                *ra = interpreter
                    .program_counter
                    .wrapping_add(Self::size() as u32) as i32;

                // Set the program counter to the new address.
                interpreter.program_counter = rs1 as u32;
            }
        } else {
            let rs2 = interpreter.registers.cpu.get(self.0.rs2)?;

            // Add
            let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs1)?;
            *rd = rd.wrapping_add(rs2);

            // Go to next instruction
            interpreter.program_counter = interpreter
                .program_counter
                .wrapping_add(Self::size() as u32);
        }

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCR},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_cebreak() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let ebreak = TypeCR { rd_rs1: 0, rs2: 0 };

        let result = CEbreakJalrAdd::decode(ebreak.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(interpreter.program_counter, 0x2);
    }

    #[test]
    fn test_cjalr() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let jalr = TypeCR { rd_rs1: 1, rs2: 0 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 4;

        let result = CEbreakJalrAdd::decode(jalr.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::RA as u8)
                .unwrap(),
            0x2
        );
        assert_eq!(interpreter.program_counter, 0x4);
    }

    #[test]
    fn test_cadd() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let add = TypeCR { rd_rs1: 1, rs2: 2 };

        *interpreter.registers.cpu.get_mut(1).unwrap() = 5;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;

        let result = CEbreakJalrAdd::decode(add.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 8);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
