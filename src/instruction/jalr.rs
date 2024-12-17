use crate::engine::{Engine, EngineState};
use crate::error::Error;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

/// Jump And Link Reg
/// Both an Opcode and an Instruction
/// Format: I-Type.
/// Action: rd = PC+4; PC = rs1 + imm
pub struct Jalr {}

impl<M: Memory> Instruction<M> for Jalr {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<EngineState, Error> {
        let inst = TypeI::from(data);

        // Get the value of the source register.
        let rs1 = engine.registers.cpu.get(inst.rs1)?;

        // Load pc + instruction size into the destination register (if not unconditional).
        if inst.rd != 0 {
            let rd = engine.registers.cpu.get_mut(inst.rd)?;
            *rd = engine.program_counter.wrapping_add(INSTRUCTION_SIZE) as i32;
        }

        // Set the program counter to the new address.
        engine.program_counter = (rs1 as u32).wrapping_add_signed(inst.imm);

        // Continue execution
        Ok(EngineState::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_jlr_negative() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let jalr = TypeI {
            funct3: 0x0,
            rd: 1,
            rs1: 2,
            imm: -0x100,
        };

        *engine.registers.cpu.get_mut(2).unwrap() = -0x200;

        let result = Jalr::decode_execute(jalr.into(), &mut engine);
        assert_eq!(result, Ok(EngineState::Running));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.program_counter, (-0x200i32 + -0x100i32) as u32);
    }

    #[test]
    fn test_jlr() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let jalr = TypeI {
            funct3: 0x0,
            rd: 1,
            rs1: 2,
            imm: 0x100,
        };

        *engine.registers.cpu.get_mut(2).unwrap() = 0x200;

        let result = Jalr::decode_execute(jalr.into(), &mut engine);
        assert_eq!(result, Ok(EngineState::Running));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.program_counter, 0x300);
    }

    #[test]
    fn test_jlr_same_reg() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let jalr = TypeI {
            funct3: 0x0,
            rd: 1,
            rs1: 1,
            imm: 0x100,
        };

        *engine.registers.cpu.get_mut(1).unwrap() = 0x200;

        let result = Jalr::decode_execute(jalr.into(), &mut engine);
        assert_eq!(result, Ok(EngineState::Running));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.program_counter, 0x300);
    }
}
