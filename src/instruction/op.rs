use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeR;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

const MUL_ADD_SUB_FUNCT3: u8 = 0b000;
const DIV_XOR_FUNCT3: u8 = 0b100;
const REM_OR_FUNCT3: u8 = 0b110;
const REMU_AND_FUNCT3: u8 = 0b111;
const MULH_SLL_FUNCT3: u8 = 0b001;
const DIVU_SRL_SRA_FUNCT3: u8 = 0b101;
const MULHSU_SLT_FUNCT3: u8 = 0b010;
const MULHU_SLTU_FUNCT3: u8 = 0b011;

#[cfg(feature = "m_extension")]
const M_EXT_FUNCT7: u8 = 0b0000001;
const SUB_SRA_FUNCT7: u8 = 0b0100000;

const ADD_FUNCT10: u16 = MUL_ADD_SUB_FUNCT3 as u16;
const SUB_FUNCT10: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNCT3 as u16;
const XOR_FUNCT10: u16 = DIV_XOR_FUNCT3 as u16;
const OR_FUNCT10: u16 = REM_OR_FUNCT3 as u16;
const AND_FUNCT10: u16 = REMU_AND_FUNCT3 as u16;
const SLL_FUNCT10: u16 = MULH_SLL_FUNCT3 as u16;
const SRL_FUNCT10: u16 = DIVU_SRL_SRA_FUNCT3 as u16;
const SRA_FUNCT10: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNCT3 as u16;
const SLT_FUNCT10: u16 = MULHSU_SLT_FUNCT3 as u16;
const SLTU_FUNCT10: u16 = MULHU_SLTU_FUNCT3 as u16;

#[cfg(feature = "m_extension")]
const MUL_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const DIV_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIV_XOR_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const REM_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REM_OR_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const REMU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REMU_AND_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const MULH_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULH_SLL_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const DIVU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const MULHSU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHSU_SLT_FUNCT3 as u16;
#[cfg(feature = "m_extension")]
const MULHU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHU_SLTU_FUNCT3 as u16;

/// Operation OpCode
/// Instructions: Add, Sub, Xor, Or, And, Sll, Srl, Sra, Slt, Sltu
/// Instructions (M Extension): Mul, Mulh, Mulhsu, Mulhu, Div, Divu, Rem, Remu
/// Format: R-Type.
pub struct Op {}

impl<M: Memory> Instruction<M> for Op {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let inst = TypeR::from(data);

        let rs1 = engine.registers.get(inst.rs1)?;
        let rs2 = engine.registers.get(inst.rs2)?;

        if inst.rd != 0 {
            // rd = 0 means its a HINT instruction, just ignore it.
            let rd = engine.registers.get_mut(inst.rd)?;
            *rd = match inst.funct10 {
                ADD_FUNCT10 => rs1.wrapping_add(rs2),        // Add
                SLL_FUNCT10 => rs1.wrapping_shl(rs2 as u32), // Sll (Logical shift left, fill with zero)
                SLT_FUNCT10 => (rs1 < rs2) as u8 as i32,     // Slt (Set less than)
                SLTU_FUNCT10 => ((rs1 as u32) < (rs2 as u32)) as u8 as i32, // Sltu (Set less than, unsigned)
                XOR_FUNCT10 => rs1 ^ rs2,                                   // Xor
                SRL_FUNCT10 => ((rs1 as u32).wrapping_shr(rs2 as u32)) as i32, // Srl (Logical shift right, fill with zero)
                OR_FUNCT10 => rs1 | rs2,                                       // Or
                AND_FUNCT10 => rs1 & rs2,                                      // And
                #[cfg(feature = "m_extension")]
                MUL_FUNCT10 => rs1.wrapping_mul(rs2), // Mul (Multiply)
                #[cfg(feature = "m_extension")]
                MULH_FUNCT10 => ((rs1 as i64).wrapping_mul(rs2 as i64) >> 32) as u32 as i32, // Mulh (Multiply High)
                #[cfg(feature = "m_extension")]
                MULHSU_FUNCT10 => {
                    ((rs1 as i64).wrapping_mul((rs2 as u32) as i64) >> 32) as u32 as i32
                } // Mulhsu (Multiply High, signed, unsigned)
                #[cfg(feature = "m_extension")]
                MULHU_FUNCT10 => ((rs1 as u32 as u64).wrapping_mul(rs2 as u32 as u64) >> 32) as i32, // Mulhu (Multiply High, unsigned)
                #[cfg(feature = "m_extension")]
                DIV_FUNCT10 => {
                    if rs2 == 0 {
                        -1
                    } else {
                        rs1.wrapping_div(rs2)
                    }
                } // Div (Divide)
                #[cfg(feature = "m_extension")]
                DIVU_FUNCT10 => {
                    if rs2 == 0 {
                        -1
                    } else {
                        (rs1 as u32).wrapping_div(rs2 as u32) as i32
                    }
                } // Divu (Divide, unsigned)
                #[cfg(feature = "m_extension")]
                REM_FUNCT10 => {
                    if rs2 == 0 {
                        rs1
                    } else {
                        rs1.wrapping_rem(rs2)
                    }
                } // Rem (Remainder)
                #[cfg(feature = "m_extension")]
                REMU_FUNCT10 => {
                    if rs2 == 0 {
                        rs1
                    } else {
                        (rs1 as u32).wrapping_rem(rs2 as u32) as i32
                    }
                } // Remu (Remainder, unsigned)
                SUB_FUNCT10 => rs1.wrapping_sub(rs2), // Sub
                SRA_FUNCT10 => rs1.wrapping_shr(rs2 as u32), // Sra (Arithmetic shift right, fill with sign bit)
                _ => return Err(EmbiveError::InvalidInstruction),
            };
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_rd_0() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 0,
            rs1: 2,
            rs2: 3,
            funct10: ADD_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 20;
        let start_regs = engine.registers;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(start_regs, engine.registers);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_add() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: ADD_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 30);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_add_wrapping() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: ADD_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = i32::MAX;
        *engine.registers.get_mut(3).unwrap() = 1;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), i32::MIN);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sub() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SUB_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 20;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 10);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sub_wrapping() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SUB_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = i32::MIN;
        *engine.registers.get_mut(3).unwrap() = 1;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), i32::MAX);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xor() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: XOR_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 0b1100;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b0110);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_or() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: OR_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 0b1100;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b1110);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_and() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: AND_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 0b1100;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b1000);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sll() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLL_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b101000);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srl() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SRL_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b10);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srl_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SRL_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0xBA987654u32 as i32;
        *engine.registers.get_mut(3).unwrap() = 28;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0xB);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sra() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SRA_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0b1010;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0b10);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sra_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SRA_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 0xBA987654u32 as i32;
        *engine.registers.get_mut(3).unwrap() = 28;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0xFFFFFFFBu32 as i32);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLT_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLT_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 20;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLT_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 20;
        *engine.registers.get_mut(3).unwrap() = 20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slt_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLT_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = -20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLTU_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLTU_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 20;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLTU_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: SLTU_FUNCT10,
        };

        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = -20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mul() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MUL_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 10;
        *engine.registers.get_mut(3).unwrap() = 20;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 200);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mul_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MUL_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = -10;
        *engine.registers.get_mut(3).unwrap() = -2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 20);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mulh() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MULH_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = i32::MAX;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (((i32::MAX as i64) * 2) >> 32) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mulhsu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MULHSU_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = i32::MAX;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (((i32::MAX as i64) * 2) >> 32) as u32 as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mulhsu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MULHSU_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = -2;
        *engine.registers.get_mut(3).unwrap() = u32::MAX as i32;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            ((-2 * (u32::MAX as i64)) >> 32) as u32 as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_mulhu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: MULHU_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = i32::MAX;
        *engine.registers.get_mut(3).unwrap() = 2;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (((i32::MAX as u64) * 2) >> 32) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_div() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: DIV_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 20;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 2);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_div_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: DIV_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = -20;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -2);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_divu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: DIVU_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = u32::MAX as i32;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.get_mut(1).unwrap(),
            (u32::MAX / 10) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_rem() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: REM_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = 101;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_rem_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: REM_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = -101;
        *engine.registers.get_mut(3).unwrap() = 10;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), -1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn test_remu() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let op = TypeR {
            rd: 1,
            rs1: 2,
            rs2: 3,
            funct10: REMU_FUNCT10,
        };
        *engine.registers.get_mut(2).unwrap() = u32::MAX as i32;
        *engine.registers.get_mut(3).unwrap() = 1;

        let result = Op::decode_execute(op.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), (u32::MAX % 1) as i32);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }
}
