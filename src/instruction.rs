//! RISC-V instruction set implementation.
#[cfg(feature = "a_extension")]
mod amo;
mod auipc;
mod branch;
mod format;
mod jal;
mod jalr;
mod load;
mod lui;
mod misc_mem;
mod op;
mod op_imm;
mod store;
mod system;

use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::memory::Memory;

#[cfg(feature = "a_extension")]
use amo::Amo;
use auipc::Auipc;
use branch::Branch;
use jal::Jal;
use jalr::Jalr;
use load::Load;
use lui::Lui;
use misc_mem::MiscMem;
use op::Op;
use op_imm::OpImm;
use store::Store;
use system::System;

/// The size of an instruction in bytes.
const INSTRUCTION_SIZE: u32 = 4;

// RISC-V opcodes.
#[cfg(feature = "a_extension")]
const AMO_OPCODE: u8 = 0b010_1111;
const LUI_OPCODE: u8 = 0b011_0111;
const AUI_PC_OPCODE: u8 = 0b001_0111;
const JAL_OPCODE: u8 = 0b110_1111;
const JALR_OPCODE: u8 = 0b110_0111;
const BRANCH_OPCODE: u8 = 0b110_0011;
const LOAD_OPCODE: u8 = 0b000_0011;
const STORE_OPCODE: u8 = 0b010_0011;
const OP_IMM_OPCODE: u8 = 0b001_0011;
const OP_OPCODE: u8 = 0b011_0011;
const MISC_MEM_OPCODE: u8 = 0b000_1111;
const SYSTEM_OPCODE: u8 = 0b111_0011;

/// Instruction trait. All instructions must implement this trait.
trait Instruction<M: Memory> {
    /// Decode and Execute the instruction.
    ///
    /// Arguments:
    /// - `data`: `u32` value representing the instruction.
    /// - `engine`: Mutable pointer to embive engine.
    ///
    /// Returns:
    /// - `Ok(bool)`: Instruction executed successfully:
    ///     - `True`: Should continue execution.
    ///     - `False`: Should halt.
    /// - `Err(EmbiveError)`: Failed to execute instruction.
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError>;
}

/// Decode and execute an instruction.
///
/// Arguments:
/// - `engine`: Mutable pointer to embive engine.
/// - `data`: `u32` value representing the instruction.
///
/// Returns:
/// - `Ok(bool)`: The instruction was decoded and executed successfully:
///     - `True`: Should continue execution.
///     - `False`: Should halt.
/// - `Err(EmbiveError)`: Failed to decode or execute instruction.
#[inline]
pub(crate) fn decode_execute<M: Memory>(
    engine: &mut Engine<M>,
    data: u32,
) -> Result<bool, EmbiveError> {
    match (data & 0x7F) as u8 {
        LOAD_OPCODE => Load::decode_execute(data, engine),
        MISC_MEM_OPCODE => MiscMem::decode_execute(data, engine),
        OP_IMM_OPCODE => OpImm::decode_execute(data, engine),
        AUI_PC_OPCODE => Auipc::decode_execute(data, engine),
        STORE_OPCODE => Store::decode_execute(data, engine),
        #[cfg(feature = "a_extension")]
        AMO_OPCODE => Amo::decode_execute(data, engine),
        OP_OPCODE => Op::decode_execute(data, engine),
        LUI_OPCODE => Lui::decode_execute(data, engine),
        BRANCH_OPCODE => Branch::decode_execute(data, engine),
        JALR_OPCODE => Jalr::decode_execute(data, engine),
        JAL_OPCODE => Jal::decode_execute(data, engine),
        SYSTEM_OPCODE => System::decode_execute(data, engine),
        _ => Err(EmbiveError::InvalidInstruction),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{engine::Engine, memory::SliceMemory};

    #[test]
    fn test_invalid_instruction() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let result = super::decode_execute(&mut engine, 0);
        assert_eq!(result, Err(EmbiveError::InvalidInstruction));
    }
}
