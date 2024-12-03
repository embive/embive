use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeR;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

const WORD_WIDTH: u8 = 0b010;

const LR_FUNCT5: u8 = 0b00010;
const SC_FUNCT5: u8 = 0b00011;
const AMOSWAP_FUNCT5: u8 = 0b00001;
const AMOADD_FUNCT5: u8 = 0b00000;
const AMOXOR_FUNCT5: u8 = 0b00100;
const AMOAND_FUNCT5: u8 = 0b01100;
const AMOOR_FUNCT5: u8 = 0b01000;
const AMOMIN_FUNCT5: u8 = 0b10000;
const AMOMAX_FUNCT5: u8 = 0b10100;
const AMOMINU_FUNCT5: u8 = 0b11000;
const AMOMAXU_FUNCT5: u8 = 0b11100;

/// Atomic Memory Operations
/// Instructions: LR, SC, AMOSWAP, AMOADD, AMOXOR, AMOAND, AMOOR, AMOMIN, AMOMAX, AMOMINU, AMOMAXU
/// Format: R-Type.
pub struct Amo {}

impl<M: Memory> Instruction<M> for Amo {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let inst = TypeR::from(data);

        let rs1 = engine.registers.get(inst.rs1)? as u32;
        let rs2 = engine.registers.get(inst.rs2)?;
        let result;

        // Check if width is supported
        match (inst.funct10 & 0b111) as u8 {
            WORD_WIDTH => {
                // 32 bits
                // Match instruction type (We ignore ordering as it isn't applicable)
                match (inst.funct10 >> 5) as u8 {
                    AMOADD_FUNCT5 => {
                        // Atomic Add (rd = mem[rs1]; mem[rs1] += rs2)
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine
                            .memory
                            .store(rs1, (result.wrapping_add(rs2)).to_le_bytes())?;
                    }
                    AMOSWAP_FUNCT5 => {
                        // Atomic Swap (rd = mem[rs1]; mem[rs1] = rs2)
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, rs2.to_le_bytes())?;
                    }
                    LR_FUNCT5 => {
                        // Load Reserved (rd = mem[rs1])
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory_reservation = Some((rs1, result)); // Reserve memory
                    }
                    SC_FUNCT5 => {
                        // Store Conditional (mem[rs1] = rs2; rd = 0 if successful, 1 otherwise)
                        match engine.memory_reservation.take() {
                            Some((addr, old_value)) => {
                                let value = i32::from_le_bytes(engine.memory.load(addr)?);
                                if value == old_value {
                                    engine.memory.store(addr, rs2.to_le_bytes())?;
                                    result = 0;
                                } else {
                                    // Value has changed
                                    result = 1;
                                }
                            }
                            None => {
                                // No reservation
                                result = 1;
                            }
                        }
                    }
                    AMOXOR_FUNCT5 => {
                        // Atomic Xor (rd = mem[rs1]; mem[rs1] ^= rs2)
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, (result ^ rs2).to_le_bytes())?;
                    }
                    AMOOR_FUNCT5 => {
                        // Atomic Or (rd = mem[rs1]; mem[rs1] |= rs2)
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, (result | rs2).to_le_bytes())?;
                    }
                    AMOAND_FUNCT5 => {
                        // Atomic And (rd = mem[rs1]; mem[rs1] &= rs2)
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, (result & rs2).to_le_bytes())?;
                    }
                    AMOMIN_FUNCT5 => {
                        // Atomic Min (rd = mem[rs1]; mem[rs1] = min(mem[rs1], rs2))
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, result.min(rs2).to_le_bytes())?;
                    }
                    AMOMAX_FUNCT5 => {
                        // Atomic Max (rd = max(mem[rs1], rs2))
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine.memory.store(rs1, result.max(rs2).to_le_bytes())?;
                    }
                    AMOMINU_FUNCT5 => {
                        // Atomic Min Unsigned (rd = minu(mem[rs1], rs2))
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine
                            .memory
                            .store(rs1, (result as u32).min(rs2 as u32).to_le_bytes())?;
                    }
                    AMOMAXU_FUNCT5 => {
                        // Atomic Max Unsigned (rd = maxu(mem[rs1], rs2))
                        result = i32::from_le_bytes(engine.memory.load(rs1)?);
                        engine
                            .memory
                            .store(rs1, (result as u32).max(rs2 as u32).to_le_bytes())?;
                    }
                    _ => return Err(EmbiveError::InvalidInstruction),
                }
            }
            _ => return Err(EmbiveError::InvalidInstruction),
        }

        // Store the result in the destination register
        if inst.rd != 0 {
            let rd = engine.registers.get_mut(inst.rd)?;
            *rd = result;
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{SliceMemory, RAM_OFFSET};

    #[test]
    fn test_amoadd() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOADD_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 2;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 16);
    }

    #[test]
    fn test_amoswap() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOSWAP_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 2;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_lr() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((LR_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 2;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(engine.memory_reservation, Some((RAM_OFFSET as u32, 14)));
    }

    #[test]
    fn test_sc() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((SC_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 2;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        engine.memory_reservation = Some((RAM_OFFSET as u32, 14));

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_amoxor() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOXOR_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 2;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 12);
    }

    #[test]
    fn test_amoor() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOOR_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 15);
    }

    #[test]
    fn test_amoand() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOAND_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_amomin() {
        let mut ram = (-14 as i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOMIN_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), -14);
    }

    #[test]
    fn test_amomax() {
        let mut ram = (-14 as i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOMAX_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), 3);
    }

    #[test]
    fn test_amominu() {
        let mut ram = (-14 as i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOMINU_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), 3);
    }

    #[test]
    fn test_amomaxu() {
        let mut ram = (-14 as i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: WORD_WIDTH as u16 | ((AMOMAXU_FUNCT5 as u16) << 5),
        };

        *engine.registers.get_mut(2).unwrap() = 3;
        *engine.registers.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = Amo::decode_execute(amo.into(), &mut engine);
        assert_eq!(result, Ok(true));

        assert_eq!(*engine.registers.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), -14);
    }
}
