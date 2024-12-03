use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

/// Miscellaneous Memory OpCode
/// Format: I-Type.
/// Action: Nothing (Not implemented / Not applicable)
pub struct MiscMem {}

impl<M: Memory> Instruction<M> for MiscMem {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let _inst = TypeI::from(data);

        // Fencing isn't applicable to this implementation.
        // This is a nop.

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
    fn test_misc_mem() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let misc_mem = TypeI {
            rd: 0,
            rs1: 0,
            imm: 0,
            funct3: 0,
        };

        let result = MiscMem::decode_execute(misc_mem.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }
}
