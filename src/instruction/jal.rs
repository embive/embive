use crate::engine::Engine;
use crate::error::Error;
use crate::instruction::format::TypeJ;
use crate::instruction::{Instruction, INSTRUCTION_SIZE};
use crate::memory::Memory;

/// Jump And Link
/// Both an Opcode and an Instruction
/// Format: J-Type.
/// Action: rd = PC+4; PC += imm
pub struct Jal {}

impl<M: Memory> Instruction<M> for Jal {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<bool, Error> {
        let inst = TypeJ::from(data);

        // Load pc + instruction size into the destination register.
        if inst.rd != 0 {
            let reg = engine.registers.cpu.get_mut(inst.rd)?;
            *reg = engine.program_counter.wrapping_add(INSTRUCTION_SIZE) as i32;
        }

        // Set the program counter to the new address.
        engine.program_counter = engine.program_counter.wrapping_add_signed(inst.imm);

        // Continue execution
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_jal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.program_counter = 0x1;
        let jal = TypeJ { rd: 1, imm: 0x1000 };

        let result = Jal::decode_execute(jal.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.cpu.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.program_counter, 0x1 + 0x1000);
    }
}
