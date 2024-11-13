mod auipc;
mod branch;
mod format;
mod load;
mod lui;
mod op;
mod op_imm;
mod store;
mod jal;
mod jalr;
mod misc_mem;
mod system;

use crate::engine::Engine;
use crate::error::EmbiveError;

use auipc::Auipc;
use branch::Branch;
use load::Load;
use lui::Lui;
use op::Op;
use op_imm::OpImm;
use store::Store;
use jal::Jal;
use jalr::Jalr;
use misc_mem::MiscMem;
use system::System;

// The size of an instruction in bytes.
const INSTRUCTION_SIZE: i32 = 4;

// RISC-V opcodes.
const LUI_OPCODE: u8 = 0b0110111;
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

/// The opcode trait. All opcodes must implement this trait.
trait Opcode {
    /// Decode the instruction.
    ///
    /// Arguments:
    /// - `data`: `u32` value representing the instruction.
    ///
    /// Returns:
    /// - `Ok(impl Instruction)`: Decoded instruction.
    /// - `Err(EmbiveError)`: Failed to decode instruction.
    fn decode(data: u32) -> Result<impl Instruction, EmbiveError>;
}

/// The instruction trait. All instructions must implement this trait.
trait Instruction {
    /// Execute the instruction.
    ///
    /// Arguments:
    ///    `engine`: Mutable pointer to virtual RISC-V engine.
    ///
    /// Returns:
    /// - `Ok(bool)`: Instruction executed successfully:
    ///     - `True`: Should continue execution.
    ///     - `False`: Should halt.
    /// - `Err(EmbiveError)`: Failed to execute instruction.
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError>;
}

/// Decodes and executes an instruction.
///
/// Arguments:
/// - `engine`: Mutable pointer to virtual RISC-V engine.
/// - `data`: `u32` value representing the instruction.
///
/// Returns:
/// - `Ok(bool)`: The instruction was decoded and executed successfully:
///     - `True`: Should continue execution.
///     - `False`: Should halt.
/// - `Err(EmbiveError)`: Failed to decode or execute instruction.
pub fn decode_and_execute(engine: &mut Engine, data: u32) -> Result<bool, EmbiveError> {
    match (data & 0x7F) as u8 {
        OP_OPCODE => Op::decode(data)?.execute(engine),
        OP_IMM_OPCODE => OpImm::decode(data)?.execute(engine),
        LOAD_OPCODE => Load::decode(data)?.execute(engine),
        STORE_OPCODE => Store::decode(data)?.execute(engine),
        BRANCH_OPCODE => Branch::decode(data)?.execute(engine),
        JAL_OPCODE => Jal::decode(data)?.execute(engine),
        JALR_OPCODE => Jalr::decode(data)?.execute(engine),
        LUI_OPCODE => Lui::decode(data)?.execute(engine),
        AUI_PC_OPCODE => Auipc::decode(data)?.execute(engine),
        MISC_MEM_OPCODE => MiscMem::decode(data)?.execute(engine),
        SYSTEM_OPCODE => System::decode(data)?.execute(engine),
        _ => Err(EmbiveError::InvalidInstruction),
    }
}
