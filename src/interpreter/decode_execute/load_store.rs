use crate::instruction::embive::LoadStore;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for LoadStore {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        let rs1 = interpreter.registers.cpu.get(inst.rs1)?;

        let address = (rs1 as u32).wrapping_add_signed(inst.imm);
        match inst.funct3 {
            Self::LB_FUNCT3 => {
                // Unwrap is safe because the slice is guaranteed to have 1 element.
                let result =
                    i8::from_le_bytes(interpreter.memory.load(address, 1)?.try_into().unwrap())
                        as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = result;
            }
            Self::LH_FUNCT3 => {
                // Unwrap is safe because the slice is guaranteed to have 2 elements
                let result =
                    i16::from_le_bytes(interpreter.memory.load(address, 2)?.try_into().unwrap())
                        as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = result;
            }
            Self::LW_FUNCT3 => {
                // Unwrap is safe because the slice is guaranteed to have 4 elements
                let result =
                    i32::from_le_bytes(interpreter.memory.load(address, 4)?.try_into().unwrap());
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = result;
            }
            Self::LBU_FUNCT3 => {
                // Unwrap is safe because the slice is guaranteed to have 1 element.
                let result =
                    u8::from_le_bytes(interpreter.memory.load(address, 1)?.try_into().unwrap())
                        as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = result;
            }
            Self::LHU_FUNCT3 => {
                // Unwrap is safe because the slice is guaranteed to have 2 elements
                let result =
                    u16::from_le_bytes(interpreter.memory.load(address, 2)?.try_into().unwrap())
                        as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = result;
            }
            Self::SB_FUNCT3 => {
                let address = (rs1 as u32).wrapping_add_signed(inst.imm);
                let rs2 = interpreter.registers.cpu.get(inst.rd_rs2)?;
                interpreter
                    .memory
                    .store(address, &(rs2 as u8).to_le_bytes())?;
            }
            Self::SH_FUNCT3 => {
                let address = (rs1 as u32).wrapping_add_signed(inst.imm);
                let rs2 = interpreter.registers.cpu.get(inst.rd_rs2)?;
                interpreter
                    .memory
                    .store(address, &(rs2 as u16).to_le_bytes())?;
            }
            Self::SW_FUNCT3 => {
                let address = (rs1 as u32).wrapping_add_signed(inst.imm);
                let rs2 = interpreter.registers.cpu.get(inst.rd_rs2)?;
                interpreter.memory.store(address, &rs2.to_le_bytes())?;
            }
            _ => return Err(Error::InvalidInstruction(data)),
        };

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        format::{Format, TypeI},
        interpreter::memory::{SliceMemory, RAM_OFFSET},
    };

    fn get_ram_addr() -> i32 {
        RAM_OFFSET as i32
    }

    #[test]
    fn test_lb() {
        let mut ram = [0x0; 2];
        ram[1] = 0x12;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lb = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LB_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lb.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x12);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lb_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lb = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LB_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lb.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -0x12);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lh() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lh = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LH_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lh.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x3412);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lh_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lh = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            funct3: LoadStore::LH_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lh.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -28098);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lw() {
        let mut ram = [0x0; 5];
        ram[1] = 0x12;
        ram[2] = 0x34;
        ram[3] = 0x56;
        ram[4] = 0x78;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LW_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lw.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x78563412);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lw_negative() {
        let mut ram = (-19088744i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            funct3: LoadStore::LW_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lw.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -19088744);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lbu() {
        let mut ram = [0x0; 2];
        ram[1] = 0x12;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lbu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LBU_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lbu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x12);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lbu_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lbu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LBU_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lbu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (-0x12i8 as u8) as i32
        );
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lhu() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lhu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            funct3: LoadStore::LHU_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lhu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x3412);
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_lhu_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let lhu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            funct3: LoadStore::LHU_FUNCT3,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode_execute(lhu.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (-28098i16 as u16) as i32
        );
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
    }

    #[test]
    fn test_sb() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let store = TypeI {
            imm: 0x1,
            funct3: LoadStore::SB_FUNCT3,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = LoadStore::decode_execute(store.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
        assert_eq!(ram[1], 0x2);
    }

    #[test]
    fn test_sh() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let store = TypeI {
            imm: 0x2,
            funct3: LoadStore::SH_FUNCT3,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::from_le(0x1234);

        let result = LoadStore::decode_execute(store.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
        assert_eq!(ram[2..4], [0x34, 0x12]);
    }

    #[test]
    fn test_sw() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let store = TypeI {
            imm: 0x0,
            funct3: LoadStore::SW_FUNCT3,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::from_le(0x12345678);

        let result = LoadStore::decode_execute(store.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::SIZE as u32);
        assert_eq!(ram[0..4], [0x78, 0x56, 0x34, 0x12]);
    }
}
