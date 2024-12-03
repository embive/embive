use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeB;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

const BEQ_FUNCT3: u8 = 0b000;
const BNE_FUNCT3: u8 = 0b001;
const BLT_FUNCT3: u8 = 0b100;
const BGE_FUNCT3: u8 = 0b101;
const BLTU_FUNCT3: u8 = 0b110;
const BGEU_FUNCT3: u8 = 0b111;

/// Branch OpCode
/// Instructions: Beq, Bne, Blt, Bqe, Bltu, Bgeu
/// Format: B-Type.
pub struct Branch {}

impl<M: Memory> Instruction<M> for Branch {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let inst = TypeB::from(data);

        let rs1 = engine.registers.get(inst.rs1)?;
        let rs2 = engine.registers.get(inst.rs2)?;

        let branch = match inst.funct3 {
            BEQ_FUNCT3 => rs1 == rs2,
            BNE_FUNCT3 => rs1 != rs2,
            BLT_FUNCT3 => rs1 < rs2,
            BGE_FUNCT3 => rs1 >= rs2,
            BLTU_FUNCT3 => (rs1 as u32) < (rs2 as u32),
            BGEU_FUNCT3 => (rs1 as u32) >= (rs2 as u32),
            _ => return Err(EmbiveError::InvalidInstruction),
        };

        engine.program_counter = if branch {
            // Branch to new address
            engine.program_counter.wrapping_add_signed(inst.imm)
        } else {
            // Go to next instruction
            engine.program_counter.wrapping_add(INSTRUCTION_SIZE)
        };

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_beq_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: -0x100,
            funct3: BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = -0x1;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1u32.wrapping_sub(0x100u32));
    }

    #[test]
    fn test_beq_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_beq_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BEQ_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bne_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BNE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bne_not_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BNE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = -0x2;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_blt_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BLT_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_blt_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BLT_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *engine.registers.get_mut(1).unwrap() = -0x2;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bge_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGE_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_bge_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_bge_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGE_FUNCT3,
            rs1: 1,
            rs2: 2,
        };
        *engine.registers.get_mut(1).unwrap() = -0x2;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bltu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BLTU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x2;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_bltu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BLTU_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *engine.registers.get_mut(1).unwrap() = -0x2;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }

    #[test]
    fn test_bgeu_greater_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGEU_FUNCT3,
            rs1: 2,
            rs2: 1,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_equal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGEU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = 0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x101);
    }

    #[test]
    fn test_bgeu_less_than() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let branch = TypeB {
            imm: 0x100,
            funct3: BGEU_FUNCT3,
            rs1: 1,
            rs2: 2,
        };

        *engine.registers.get_mut(1).unwrap() = 0x1;
        *engine.registers.get_mut(2).unwrap() = -0x1;

        let result = Branch::decode_execute(branch.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }
}
