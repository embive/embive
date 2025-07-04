use crate::instruction::embive::CLw;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CLw {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Load word from memory
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;
        let address = (rs1 as u32).wrapping_add(self.0.imm as u32);

        // Unwrap is safe because the slice is guaranteed to have 4 elements
        let result = i32::from_le_bytes(interpreter.memory.load(address, 4)?.try_into().unwrap());
        // Store the result in the destination register
        let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
        *rd = result;

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
        format::{Format, TypeCL},
        instruction::embive::InstructionImpl,
        interpreter::memory::{SliceMemory, RAM_OFFSET},
    };

    use super::*;

    fn get_ram_addr() -> i32 {
        RAM_OFFSET as i32
    }

    #[test]
    fn test_clw() {
        let mut ram = [0x0; 8];
        ram[4] = 0x12;
        ram[5] = 0x34;
        ram[6] = 0x56;
        ram[7] = 0x78;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lw = TypeCL {
            rd_rs2: 8,
            rs1: 9,
            imm: 0x4,
        };
        *interpreter.registers.cpu.get_mut(9).unwrap() = get_ram_addr();

        let result = CLw::decode(lw.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(8).unwrap(), 0x78563412);
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
