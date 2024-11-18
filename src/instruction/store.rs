use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeS;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};

const SB_FUNCT3: u8 = 0b000;
const SH_FUNCT3: u8 = 0b001;
const SW_FUNCT3: u8 = 0b010;

/// Store OpCode
/// Instructions: Sb, Sh, Sw
/// Format: S-Type.
pub struct Store {
    ty: TypeS,
}

impl Opcode for Store {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction {
        Self {
            ty: TypeS::from(data),
        }
    }
}

impl Instruction for Store {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        let rs1 = engine.registers.get(self.ty.rs1)?;
        let rs2 = engine.registers.get(self.ty.rs2)?;

        let address = (rs1 as u32).wrapping_add_signed(self.ty.imm);
        match self.ty.funct3 {
            SB_FUNCT3 => engine.memory.store(address, (rs2 as u8).to_le_bytes())?,
            SH_FUNCT3 => engine.memory.store(address, (rs2 as u16).to_le_bytes())?,
            SW_FUNCT3 => engine.memory.store(address, rs2.to_le_bytes())?,
            _ => return Err(EmbiveError::InvalidInstruction),
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{engine::initial_program_counter, memory::RAM_OFFSET};

    fn get_ram_addr() -> i32 {
        RAM_OFFSET as i32
    }

    #[test]
    fn test_sb() {
        let mut memory = [0; 2];
        let mut engine = Engine::new(&[], &mut memory, Default::default()).unwrap();
        let store = Store {
            ty: TypeS {
                imm: 0x1,
                funct3: SB_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.registers.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.get_mut(2).unwrap() = 0x2;

        let result = store.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            engine.program_counter,
            initial_program_counter() + INSTRUCTION_SIZE
        );
        assert_eq!(memory[1], 0x2);
    }

    #[test]
    fn test_sh() {
        let mut memory = [0; 4];
        let mut engine = Engine::new(&[], &mut memory, Default::default()).unwrap();
        let store = Store {
            ty: TypeS {
                imm: 0x2,
                funct3: SH_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.registers.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.get_mut(2).unwrap() = 0x1234;

        let result = store.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            engine.program_counter,
            initial_program_counter() + INSTRUCTION_SIZE
        );
        assert_eq!(memory[2..4], [0x34, 0x12]);
    }

    #[test]
    fn test_sw() {
        let mut memory = [0; 4];
        let mut engine = Engine::new(&[], &mut memory, Default::default()).unwrap();
        let store = Store {
            ty: TypeS {
                imm: 0x0,
                funct3: SW_FUNCT3,
                rs1: 1,
                rs2: 2,
            },
        };

        *engine.registers.get_mut(1).unwrap() = get_ram_addr();
        *engine.registers.get_mut(2).unwrap() = 0x12345678;

        let result = store.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            engine.program_counter,
            initial_program_counter() + INSTRUCTION_SIZE
        );
        assert_eq!(memory[0..4], [0x78, 0x56, 0x34, 0x12]);
    }
}
