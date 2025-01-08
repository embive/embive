use crate::instruction::embive::CSw;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CSw {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Store word on memory
        let rs1 = interpreter.registers.cpu.get(inst.rs1)?;
        let address = (rs1 as u32).wrapping_add(inst.imm as u32);

        let rs2: &mut i32 = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
        interpreter.memory.store(address, &rs2.to_le_bytes())?;

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCL, COMPRESSED_REGISTER_OFFSET},
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
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lw = TypeCL {
            rd_rs2: COMPRESSED_REGISTER_OFFSET,
            rs1: COMPRESSED_REGISTER_OFFSET + 1,
            imm: 0x4,
        };
        *interpreter
            .registers
            .cpu
            .get_mut(COMPRESSED_REGISTER_OFFSET + 1)
            .unwrap() = get_ram_addr();
        *interpreter
            .registers
            .cpu
            .get_mut(COMPRESSED_REGISTER_OFFSET)
            .unwrap() = i32::from_le(0x78563412);

        let result = CSw::decode_execute(lw.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x2);
        assert_eq!(&ram[4..], &[0x12, 0x34, 0x56, 0x78]);
    }
}
