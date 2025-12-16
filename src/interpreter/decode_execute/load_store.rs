use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::LoadStore;
use crate::interpreter::{
    memory::{Memory, MemoryType},
    Error, Interpreter, State,
};

use super::Execute;

impl<M: Memory> Execute<M> for LoadStore {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let rs1 = interpreter.registers.cpu.get(self.0.rs1)?;

        let address = (rs1 as u32).wrapping_add_signed(self.0.imm);
        match self.0.func {
            Self::LB_FUNC => {
                let result = i8::load(interpreter.memory, address)? as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = result;
            }
            Self::LH_FUNC => {
                let result = i16::load(interpreter.memory, address)? as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = result;
            }
            Self::LW_FUNC => {
                let result = i32::load(interpreter.memory, address)?;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = result;
            }
            Self::LBU_FUNC => {
                let result = u8::load(interpreter.memory, address)? as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = result;
            }
            Self::LHU_FUNC => {
                let result = u16::load(interpreter.memory, address)? as i32;
                // Store the result in the destination register
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = result;
            }
            Self::SB_FUNC => {
                let address = (rs1 as u32).wrapping_add_signed(self.0.imm);
                let rs2 = interpreter.registers.cpu.get(self.0.rd_rs2)?;
                (rs2 as u8).store(interpreter.memory, address)?;
            }
            Self::SH_FUNC => {
                let address = (rs1 as u32).wrapping_add_signed(self.0.imm);
                let rs2 = interpreter.registers.cpu.get(self.0.rd_rs2)?;
                (rs2 as u16).store(interpreter.memory, address)?;
            }
            Self::SW_FUNC => {
                let address = (rs1 as u32).wrapping_add_signed(self.0.imm);
                let rs2 = interpreter.registers.cpu.get(self.0.rd_rs2)?;
                rs2.store(interpreter.memory, address)?;
            }
            _ => return Err(Error::InvalidInstruction(interpreter.program_counter)),
        };

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        format::{Format, TypeI},
        instruction::embive::InstructionImpl,
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lb = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LB_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lb.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x12);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lb_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lb = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LB_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lb.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -0x12);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lh() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lh = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LH_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lh.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x3412);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lh_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lh = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            func: LoadStore::LH_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lh.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -28098);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lw() {
        let mut ram = [0x0; 5];
        ram[1] = 0x12;
        ram[2] = 0x34;
        ram[3] = 0x56;
        ram[4] = 0x78;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LW_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lw.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x78563412);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lw_negative() {
        let mut ram = (-19088744i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            func: LoadStore::LW_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lw.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -19088744);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lbu() {
        let mut ram = [0x0; 2];
        ram[1] = 0x12;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lbu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LBU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lbu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x12);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lbu_negative() {
        let mut ram = [0x0; 2];
        ram[1] = -0x12i8 as u8;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lbu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LBU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lbu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (-0x12i8 as u8) as i32
        );
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lhu() {
        let mut ram = [0x0; 3];
        ram[1] = 0x12;
        ram[2] = 0x34;

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lhu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x1,
            func: LoadStore::LHU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lhu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0x3412);
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_lhu_negative() {
        let mut ram = (-28098i16).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let lhu = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x0,
            func: LoadStore::LHU_FUNC,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = get_ram_addr();

        let result = LoadStore::decode(lhu.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (-28098i16 as u16) as i32
        );
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
    }

    #[test]
    fn test_sb() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let store = TypeI {
            imm: 0x1,
            func: LoadStore::SB_FUNC,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x2;

        let result = LoadStore::decode(store.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
        assert_eq!(ram[1], 0x2);
    }

    #[test]
    fn test_sh() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let store = TypeI {
            imm: 0x2,
            func: LoadStore::SH_FUNC,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::from_le(0x1234);

        let result = LoadStore::decode(store.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
        assert_eq!(ram[2..4], [0x34, 0x12]);
    }

    #[test]
    fn test_sw() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let store = TypeI {
            imm: 0x0,
            func: LoadStore::SW_FUNC,
            rs1: 1,
            rd_rs2: 2,
        };

        *interpreter.registers.cpu.get_mut(1).unwrap() = get_ram_addr();
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::from_le(0x12345678);

        let result = LoadStore::decode(store.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, LoadStore::size() as u32);
        assert_eq!(ram[0..4], [0x78, 0x56, 0x34, 0x12]);
    }
}
