use crate::engine::Engine;
use crate::error::Error;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

const ADDI_FUNC3: u8 = 0b000;
const XORI_FUNC3: u8 = 0b100;
const ORI_FUNC3: u8 = 0b110;
const ANDI_FUNC3: u8 = 0b111;
const SLLI_FUNC3: u8 = 0b001;
const SRLI_SRAI_FUNC3: u8 = 0b101;
const SLTI_FUNC3: u8 = 0b010;
const SLTIU_FUNC3: u8 = 0b011;

/// Operation Immediate OpCode
/// Instructions: Addi, Xori, Ori, Andi, Slli, Srli, Srai, Slti, Sltiu
/// Format: I-Type.
pub struct OpImm {}

impl<M: Memory> Instruction<M> for OpImm {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<bool, Error> {
        let inst = TypeI::from(data);

        let rs1 = engine.registers.cpu.get(inst.rs1)?;
        let imm = inst.imm;

        if inst.rd != 0 {
            // rd = 0 means its a HINT instruction, just ignore it.
            let rd = engine.registers.cpu.get_mut(inst.rd)?;
            *rd = match inst.funct3 {
                ADDI_FUNC3 => rs1.wrapping_add(imm),
                SLLI_FUNC3 => rs1 << (imm & 0b11111),
                SLTI_FUNC3 => (rs1 < imm) as u8 as i32,
                SLTIU_FUNC3 => ((rs1 as u32) < (imm as u32)) as u8 as i32,
                XORI_FUNC3 => rs1 ^ imm,
                SRLI_SRAI_FUNC3 => {
                    if (imm & (0b1 << 10)) != 0 {
                        // Sra (Arithmetic shift right, fill with sign bit)
                        rs1 >> (imm & 0b11111)
                    } else {
                        // Srl (Logical shift right, fill with zero)
                        ((rs1 as u32) >> ((imm & 0b11111) as u32)) as i32
                    }
                }
                ORI_FUNC3 => rs1 | imm,
                ANDI_FUNC3 => rs1 & imm,
                _ => return Err(Error::InvalidInstruction),
            };
        }

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_addi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let addi = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x100,
            funct3: ADDI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode_execute(addi.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x101);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_addi_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let addi = TypeI {
            rd: 1,
            rs1: 2,
            imm: -100,
            funct3: ADDI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 1;

        let result = OpImm::decode_execute(addi.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), -99);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let xori = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x100,
            funct3: XORI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x123;

        let result = OpImm::decode_execute(xori.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x023);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_xori_not() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let xori = TypeI {
            rd: 1,
            rs1: 2,
            imm: -1,
            funct3: XORI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(xori.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), !0x1234);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ori() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let ori = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x100,
            funct3: ORI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(ori.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x1234 | 0x100);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ori_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let ori = TypeI {
            rd: 1,
            rs1: 2,
            imm: -0x100,
            funct3: ORI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(ori.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x1234 | -0x100);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_andi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let andi = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x100,
            funct3: ANDI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(andi.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x1234 & 0x100);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slli = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0b101,
            funct3: SLLI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(slli.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x1234 << 0b101);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srli() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let srli = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0b101,
            funct3: SRLI_SRAI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(srli.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(
            *engine.registers.cpu.get_mut(1).unwrap(),
            ((-0x1234i32 as u32) >> 0b101) as i32
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_srai() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let srai = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0b101 | (0b1 << 10),
            funct3: SRLI_SRAI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(srai.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), -0x1234 >> 0b101);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x123,
            funct3: SLTI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode_execute(slti.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: SLTI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(slti.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: SLTI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode_execute(slti.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_slti_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let slti = TypeI {
            rd: 1,
            rs1: 2,
            imm: -0x1000,
            funct3: SLTI_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(slti.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_lower() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x123,
            funct3: SLTIU_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x100;

        let result = OpImm::decode_execute(sltiu.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_greater() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: SLTIU_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let result = OpImm::decode_execute(sltiu.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x1000,
            funct3: SLTIU_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1000;

        let result = OpImm::decode_execute(sltiu.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_sltiu_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let sltiu = TypeI {
            rd: 1,
            rs1: 2,
            imm: -0x100,
            funct3: SLTIU_FUNC3,
        };
        *engine.registers.cpu.get_mut(2).unwrap() = -0x1234;

        let result = OpImm::decode_execute(sltiu.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 1);
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }
}
