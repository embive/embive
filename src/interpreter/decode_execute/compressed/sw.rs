use crate::instruction::embive::CSw;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CSw {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Store word on memory
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;
        let address = (rs1 as u32).wrapping_add(self.0.imm as u32);

        let rs2 = interpreter.registers.cpu.get(self.0.rd_rs2)?;
        interpreter.memory.store(address, &rs2.to_le_bytes())?;

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
    fn test_csw() {
        let mut ram = [0x0; 8];

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let lw = TypeCL {
            rd_rs2: 8,
            rs1: 9,
            imm: 0x4,
        };
        *interpreter.registers.cpu.get_mut(9).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(8).unwrap() = i32::from_le(0x78563412);

        let result = CSw::decode(lw.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x2);
        assert_eq!(&ram[4..], &[0x12, 0x34, 0x56, 0x78]);
    }
}
