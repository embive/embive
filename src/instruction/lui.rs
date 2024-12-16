use crate::engine::Engine;
use crate::error::Error;
use crate::instruction::format::TypeU;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

/// Load Upper Immediate
/// Both an Opcode and an Instruction
/// Format: U-Type.
/// Action: rd = imm
pub struct Lui {}

impl<M: Memory> Instruction<M> for Lui {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<bool, Error> {
        let inst = TypeU::from(data);

        if inst.rd != 0 {
            // rd = 0 means its a HINT instruction, just ignore it.
            // Load the immediate value into the register.
            let reg = engine.registers.cpu.get_mut(inst.rd)?;
            *reg = inst.imm;
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
    fn test_lui() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let lui = TypeU { rd: 1, imm: 0x1000 };

        let result = Lui::decode_execute(lui.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x1000);
        assert_eq!(engine.program_counter, 0x1 + INSTRUCTION_SIZE);
    }
}
