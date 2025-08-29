use crate::instruction::embive::CSwsp;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CSwsp {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Store word to memory (sp + imm)
        let sp = interpreter.registers.cpu.get(CPURegister::SP as u8)?;
        let address = (sp as u32).wrapping_add(self.0.imm as u32);

        let rs2 = interpreter.registers.cpu.get_mut(self.0.rs2)?;
        interpreter
            .memory
            .store_bytes(address, &rs2.to_le_bytes())?;

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCSS},
        instruction::embive::InstructionImpl,
        interpreter::memory::{SliceMemory, RAM_OFFSET},
    };

    use super::*;

    fn get_ram_addr() -> i32 {
        RAM_OFFSET as i32
    }

    #[test]
    fn test_csw() {
        let mut ram = [0x0; 8];

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let swsp = TypeCSS { rs2: 1, imm: 0x4 };

        *interpreter
            .registers
            .cpu
            .get_mut(CPURegister::SP as u8)
            .unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(1).unwrap() = i32::from_le(0x78563412);

        let result = CSwsp::decode(swsp.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x2);
        assert_eq!(&ram[4..], &[0x12, 0x34, 0x56, 0x78]);
    }
}
