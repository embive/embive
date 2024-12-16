use crate::engine::Engine;
use crate::error::Error;
use crate::instruction::format::TypeS;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

const SB_FUNCT3: u8 = 0b000;
const SH_FUNCT3: u8 = 0b001;
const SW_FUNCT3: u8 = 0b010;

/// Store OpCode
/// Instructions: Sb, Sh, Sw
/// Format: S-Type.
pub struct Store {}

impl<M: Memory> Instruction<M> for Store {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<bool, Error> {
        let inst = TypeS::from(data);

        let rs1 = engine.registers.cpu.get(inst.rs1)?;
        let rs2 = engine.registers.cpu.get(inst.rs2)?;

        let address = (rs1 as u32).wrapping_add_signed(inst.imm);
        match inst.funct3 {
            SB_FUNCT3 => engine.memory.store(address, (rs2 as u8).to_le_bytes())?,
            SH_FUNCT3 => engine.memory.store(address, (rs2 as u16).to_le_bytes())?,
            SW_FUNCT3 => engine.memory.store(address, rs2.to_le_bytes())?,
            _ => return Err(Error::InvalidInstruction),
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{SliceMemory, RAM_OFFSET};

    fn get_ram_addr() -> i32 {
        RAM_OFFSET as i32
    }

    #[test]
    fn test_sb() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let store = TypeS {
            imm: 0x1,
            funct3: SB_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = Store::decode_execute(store.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
        assert_eq!(ram[1], 0x2);
    }

    #[test]
    fn test_sh() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let store = TypeS {
            imm: 0x2,
            funct3: SH_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = Store::decode_execute(store.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
        assert_eq!(ram[2..4], [0x34, 0x12]);
    }

    #[test]
    fn test_sw() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let store = TypeS {
            imm: 0x0,
            funct3: SW_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.cpu.get_mut(2).unwrap() = 0x12345678;

        let result = Store::decode_execute(store.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
        assert_eq!(ram[0..4], [0x78, 0x56, 0x34, 0x12]);
    }
}
