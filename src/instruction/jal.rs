use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeJ;
use crate::instruction::{Instruction, Opcode, INSTRUCTION_SIZE};
use crate::memory::Memory;

/// Jump And Link
/// Both an Opcode and an Instruction
/// Format: J-Type.
/// Action: rd = PC+4; PC += imm
pub struct Jal {
    ty: TypeJ,
}

impl<M: Memory> Opcode<M> for Jal {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction<M> {
        Self {
            ty: TypeJ::from(data),
        }
    }
}

impl<M: Memory> Instruction<M> for Jal {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        // Load pc + instruction size into the destination register.
        if self.ty.rd != 0 {
            let reg = engine.registers.get_mut(self.ty.rd)?;
            *reg = engine.program_counter.wrapping_add(INSTRUCTION_SIZE) as i32;
        }

        // Set the program counter to the new address.
        engine.program_counter = engine.program_counter.wrapping_add_signed(self.ty.imm);

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
        let jal = Jal {
            ty: TypeJ { rd: 1, imm: 0x1000 },
        };

        let result = jal.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(*engine.registers.get_mut(1).unwrap(), 0x5);
        assert_eq!(engine.program_counter, 0x1 + 0x1000);
    }
}
