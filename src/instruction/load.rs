use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};
use crate::memory::Memory;

const LB_FUNCT3: u8 = 0b000;
const LH_FUNCT3: u8 = 0b001;
const LW_FUNCT3: u8 = 0b010;
const LBU_FUNCT3: u8 = 0b100;
const LHU_FUNCT3: u8 = 0b101;

/// Load OpCode
/// Instructions: Lb, Lh, Lw, Lbu, Lhu
/// Format: I-Type.
pub struct Load {
    ty: TypeI,
}

impl<M: Memory> Opcode<M> for Load {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction<M> {
        Self {
            ty: TypeI::from(data),
        }
    }
}

impl<M: Memory> Instruction<M> for Load {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let rs1 = engine.registers.get(self.ty.rs1)?;

        let address = (rs1 as u32).wrapping_add_signed(self.ty.imm);
        let result = match self.ty.funct3 {
            LB_FUNCT3 => i8::from_le_bytes(engine.memory.load(address)?) as i32,
            LH_FUNCT3 => i16::from_le_bytes(engine.memory.load(address)?) as i32,
            LW_FUNCT3 => i32::from_le_bytes(engine.memory.load(address)?) as i32,
            LBU_FUNCT3 => u8::from_le_bytes(engine.memory.load(address)?) as i32,
            LHU_FUNCT3 => u16::from_le_bytes(engine.memory.load(address)?) as i32,
            _ => return Err(EmbiveError::InvalidInstruction),
        };

        // Store the result in the destination register
        let rd = engine.registers.get_mut(self.ty.rd)?;
        *rd = result;

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
    fn test_lb() {
        let mut ram = [0x0; 2];
        ram[1] = 0x12;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lb = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LB_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lb.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x12);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lb_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lb = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LB_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lb.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -0x12);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lh() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lh = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LH_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lh.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x3412);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lh_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lh = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LH_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lh.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -28098);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lw() {
        let mut ram = [0x0; 5];
        ram[1] = 0x12;
        ram[2] = 0x34;
        ram[3] = 0x56;
        ram[4] = 0x78;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lw = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LW_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lw.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x78563412);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lw_negative() {
        let mut ram = (-19088744i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lw = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LW_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lw.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -19088744);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lbu() {
        let mut ram = [0x0; 2];
        ram[1] = 0x12;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lbu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LBU_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lbu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x12);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lbu_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lbu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LBU_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lbu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (-0x12i8 as u8) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lhu() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lhu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x1,
                funct3: LHU_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lhu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x3412);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_lhu_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let lhu = Load {
            ty: TypeI {
                rd: 1,
                rs1: 2,
                imm: 0x0,
                funct3: LHU_FUNCT3,
            },
        };
        *engine.registers.get_mut(2).unwrap() = get_ram_addr();

        let result = lhu.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (-28098i16 as u16) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }
}
