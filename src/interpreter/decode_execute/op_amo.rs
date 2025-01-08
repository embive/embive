use crate::instruction::embive::OpAmo;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for OpAmo {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        let rs1 = interpreter.registers.cpu.get(inst.rs1)?;
        let rs2 = interpreter.registers.cpu.get(inst.rs2)?;

        let result = match inst.funct10 {
            Self::ADD_FUNCT10 => rs1.wrapping_add(rs2),        // Add
            Self::SUB_FUNCT10 => rs1.wrapping_sub(rs2),        // Sub
            Self::SLL_FUNCT10 => rs1.wrapping_shl(rs2 as u32), // Sll (Logical shift left, fill with zero)
            Self::SLT_FUNCT10 => (rs1 < rs2) as u8 as i32,     // Slt (Set less than)
            Self::SLTU_FUNCT10 => ((rs1 as u32) < (rs2 as u32)) as u8 as i32, // Sltu (Set less than, unsigned)
            Self::XOR_FUNCT10 => rs1 ^ rs2,                                   // Xor
            Self::SRL_FUNCT10 => ((rs1 as u32).wrapping_shr(rs2 as u32)) as i32, // Srl (Logical shift right, fill with zero)
            Self::SRA_FUNCT10 => rs1.wrapping_shr(rs2 as u32), // Sra (Arithmetic shift right, fill with sign bit)
            Self::OR_FUNCT10 => rs1 | rs2,                     // Or
            Self::AND_FUNCT10 => rs1 & rs2,                    // And
            Self::MUL_FUNCT10 => rs1.wrapping_mul(rs2),        // Mul (Multiply)
            Self::MULH_FUNCT10 => ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32 as i32, // Mulh (Multiply High)
            Self::MULHSU_FUNCT10 => {
                ((rs1 as i64).wrapping_mul((rs2 as u32) as i64) >> 32) as u32 as i32
            } // Mulhsu (Multiply High, signed, unsigned)
            Self::MULHU_FUNCT10 => {
                ((rs1 as u32 as u64).wrapping_mul(rs2 as u32 as u64) >> 32) as i32
            } // Mulhu (Multiply High, unsigned)
            Self::DIV_FUNCT10 => {
                if rs2 == 0 {
                    -1
                } else {
                    rs1.wrapping_div(rs2)
                }
            } // Div (Divide)
            Self::DIVU_FUNCT10 => {
                if rs2 == 0 {
                    -1
                } else {
                    (rs1 as u32).wrapping_div(rs2 as u32) as i32
                }
            } // Divu (Divide, unsigned)
            Self::REM_FUNCT10 => {
                if rs2 == 0 {
                    rs1
                } else {
                    rs1.wrapping_rem(rs2)
                }
            } // Rem (Remainder)
            Self::REMU_FUNCT10 => {
                if rs2 == 0 {
                    rs1
                } else {
                    (rs1 as u32).wrapping_rem(rs2 as u32) as i32
                }
            } // Remu (Remainder, unsigned)
            _ => {
                // Atomic operations
                let value = i32::from_le_bytes(
                    // Unwrap is safe because the slice is guaranteed to have 4 elements.
                    interpreter.memory.load(rs1 as u32, 4)?.try_into().unwrap(),
                );

                match inst.funct10 {
                    Self::LR_FUNCT10 => {
                        // Load Reserved (rd = mem[rs1])
                        interpreter.memory_reservation = Some((rs1 as u32, value)); // Reserve memory
                        value
                    }
                    Self::SC_FUNCT10 => {
                        // Store Conditional (mem[rs1] = rs2; rd = 0 if successful, 1 otherwise)
                        let ret;
                        match interpreter.memory_reservation.take() {
                            Some((addr, old_value)) => {
                                if addr == rs1 as u32 && value == old_value {
                                    interpreter.memory.store(addr, &rs2.to_le_bytes())?;
                                    ret = 0;
                                } else {
                                    // Value has changed or address is different
                                    ret = 1;
                                }
                            }
                            None => {
                                // No reservation
                                ret = 1;
                            }
                        }
                        ret
                    }
                    Self::AMOSWAP_FUNCT10 => {
                        // Atomic Swap (rd = mem[rs1]; mem[rs1] = rs2)
                        interpreter.memory.store(rs1 as u32, &rs2.to_le_bytes())?;
                        value
                    }
                    Self::AMOADD_FUNCT10 => {
                        // Atomic Add (rd = mem[rs1]; mem[rs1] += rs2)
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value.wrapping_add(rs2)).to_le_bytes())?;
                        value
                    }
                    Self::AMOXOR_FUNCT10 => {
                        // Atomic Xor (rd = mem[rs1]; mem[rs1] ^= rs2)
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value ^ rs2).to_le_bytes())?;
                        value
                    }
                    Self::AMOAND_FUNCT10 => {
                        // Atomic And (rd = mem[rs1]; mem[rs1] &= rs2)
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value & rs2).to_le_bytes())?;
                        value
                    }
                    Self::AMOOR_FUNCT10 => {
                        // Atomic Or (rd = mem[rs1]; mem[rs1] |= rs2)
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value | rs2).to_le_bytes())?;
                        value
                    }
                    Self::AMOMIN_FUNCT10 => {
                        // Atomic Min (rd = mem[rs1]; mem[rs1] = min(mem[rs1], rs2))
                        interpreter
                            .memory
                            .store(rs1 as u32, &value.min(rs2).to_le_bytes())?;
                        value
                    }
                    Self::AMOMAX_FUNCT10 => {
                        // Atomic Max (rd = max(mem[rs1], rs2))
                        interpreter
                            .memory
                            .store(rs1 as u32, &value.max(rs2).to_le_bytes())?;
                        value
                    }
                    Self::AMOMINU_FUNCT10 => {
                        // Atomic Min Unsigned (rd = minu(mem[rs1], rs2))
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value as u32).min(rs2 as u32).to_le_bytes())?;
                        value
                    }
                    Self::AMOMAXU_FUNCT10 => {
                        // Atomic Max Unsigned (rd = maxu(mem[rs1], rs2))
                        interpreter
                            .memory
                            .store(rs1 as u32, &(value as u32).max(rs2 as u32).to_le_bytes())?;
                        value
                    }
                    _ => return Err(Error::InvalidInstruction(data)),
                }
            }
        };

        if inst.rd != 0 {
            let rd = interpreter.registers.cpu.get_mut(inst.rd)?;
            *rd = result;
        }

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeR},
        interpreter::memory::{SliceMemory, RAM_OFFSET},
    };

    use super::*;

    #[test]
    fn test_rd_0() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 0,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::ADD_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;
        let start_regs = interpreter.registers;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(start_regs, interpreter.registers);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_add() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::ADD_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 30);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_add_wrapping() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::ADD_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::MAX;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 1;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), i32::MIN);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sub() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SUB_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 10);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sub_wrapping() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SUB_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::MIN;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 1;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), i32::MAX);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_xor() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::XOR_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 0b1100;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b0110);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_or() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::OR_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 0b1100;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b1110);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_and() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::AND_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 0b1100;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b1000);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sll() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLL_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b101000);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_srl() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SRL_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b10);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_srl_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SRL_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0xBA987654u32 as i32;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 28;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0xB);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sra() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SRA_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0b1010;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0b10);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sra_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SRA_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 0xBA987654u32 as i32;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 28;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            0xFFFFFFFBu32 as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_slt_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLT_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_slt_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLT_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_slt_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLT_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_slt_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLT_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = -20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sltu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLTU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sltu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLTU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sltu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLTU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_sltu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::SLTU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = -20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mul() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MUL_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 20;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 200);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mul_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MUL_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -10;
        *interpreter.registers.cpu.get_mut(3).unwrap() = -2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 20);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mulh() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MULH_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::MAX;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (((i32::MAX as i64) * 2) >> 32) as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mulhsu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MULHSU_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::MAX;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (((i32::MAX as i64) * 2) >> 32) as u32 as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mulhsu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MULHSU_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = u32::MAX as i32;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            ((-2 * (u32::MAX as i64)) >> 32) as u32 as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_mulhu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::MULHU_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = i32::MAX;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 2;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (((i32::MAX as u64) * 2) >> 32) as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_div() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::DIV_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 2);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_div_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::DIV_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -20;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -2);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_divu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::DIVU_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = u32::MAX as i32;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(
            *interpreter.registers.cpu.get_mut(1).unwrap(),
            (u32::MAX / 10) as i32
        );
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_rem() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::REM_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = 101;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_rem_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::REM_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = -101;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 10;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -1);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_remu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OpAmo::REMU_FUNCT10,
        };
        *interpreter.registers.cpu.get_mut(2).unwrap() = u32::MAX as i32;
        *interpreter.registers.cpu.get_mut(3).unwrap() = 1;

        let result = OpAmo::decode_execute(op.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(interpreter.program_counter, OpAmo::SIZE as u32);
    }

    #[test]
    fn test_amoadd() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOADD_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 16);
    }

    #[test]
    fn test_amoswap() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOSWAP_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_lr() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::LR_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(interpreter.memory_reservation, Some((RAM_OFFSET, 14)));
    }

    #[test]
    fn test_sc() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::SC_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        interpreter.memory_reservation = Some((RAM_OFFSET, 14));

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_amoxor() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOXOR_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 2;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 12);
    }

    #[test]
    fn test_amoor() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOOR_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 15);
    }

    #[test]
    fn test_amoand() {
        let mut ram = 14i32.to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOAND_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), 14);
        assert_eq!(i32::from_le_bytes(ram), 2);
    }

    #[test]
    fn test_amomin() {
        let mut ram = (-14_i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOMIN_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), -14);
    }

    #[test]
    fn test_amomax() {
        let mut ram = (-14_i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOMAX_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), 3);
    }

    #[test]
    fn test_amominu() {
        let mut ram = (-14_i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOMINU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), 3);
    }

    #[test]
    fn test_amomaxu() {
        let mut ram = (-14_i32).to_le_bytes();

        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();

        let amo = TypeR {
            rd: 1,
            rs1: 3,
            rs2: 2,
            funct10: OpAmo::AMOMAXU_FUNCT10,
        };

        *interpreter.registers.cpu.get_mut(2).unwrap() = 3;
        *interpreter.registers.cpu.get_mut(3).unwrap() = RAM_OFFSET as i32;

        let result = OpAmo::decode_execute(amo.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));

        assert_eq!(*interpreter.registers.cpu.get_mut(1).unwrap(), -14);
        assert_eq!(i32::from_le_bytes(ram), -14);
    }
}
