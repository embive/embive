//! Instruction decoding and execution module.
mod auipc;
mod branch;
mod compressed;
mod jal;
mod jalr;
mod load_store;
mod lui;
mod op_amo;
mod op_imm;
mod system_misc_mem;

use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use crate::instruction::embive;

/// DecodeExecute trait. All instructions must implement this trait.
trait DecodeExecute<M: Memory> {
    /// Decode and Execute the instruction.
    ///
    /// Arguments:
    /// - `data`: `u32` value representing the instruction.
    /// - `interpreter`: Mutable pointer to embive interpreter.
    ///
    /// Returns:
    /// - `Ok(EngineState)`: Instruction executed successfully.
    /// - `Err(Error)`: Failed to execute instruction.
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error>;
}

/// Decode and execute an instruction.
///
/// Arguments:
/// - `interpreter`: Mutable pointer to embive interpreter.
/// - `data`: `u32` value representing the instruction.
///
/// Returns:
/// - `Ok(EngineState)`: The instruction was decoded and executed successfully.
/// - `Err(Error)`: Failed to decode or execute instruction.
#[inline(always)]
pub fn decode_execute<M: Memory>(
    interpreter: &mut Interpreter<'_, M>,
    data: u32,
) -> Result<State, Error> {
    // First 5 bits are the opcode.
    match (data & 0x1F) as u8 {
        embive::CAddi4spn::OPCODE => embive::CAddi4spn::decode_execute(data, interpreter),
        embive::CLw::OPCODE => embive::CLw::decode_execute(data, interpreter),
        embive::CSw::OPCODE => embive::CSw::decode_execute(data, interpreter),
        embive::CAddi::OPCODE => embive::CAddi::decode_execute(data, interpreter),
        embive::CJal::OPCODE => embive::CJal::decode_execute(data, interpreter),
        embive::CLi::OPCODE => embive::CLi::decode_execute(data, interpreter),
        embive::CAddi16sp::OPCODE => embive::CAddi16sp::decode_execute(data, interpreter),
        embive::CLui::OPCODE => embive::CLui::decode_execute(data, interpreter),
        embive::CSrli::OPCODE => embive::CSrli::decode_execute(data, interpreter),
        embive::CSrai::OPCODE => embive::CSrai::decode_execute(data, interpreter),
        embive::CAndi::OPCODE => embive::CAndi::decode_execute(data, interpreter),
        embive::CSub::OPCODE => embive::CSub::decode_execute(data, interpreter),
        embive::CXor::OPCODE => embive::CXor::decode_execute(data, interpreter),
        embive::COr::OPCODE => embive::COr::decode_execute(data, interpreter),
        embive::CAnd::OPCODE => embive::CAnd::decode_execute(data, interpreter),
        embive::CJ::OPCODE => embive::CJ::decode_execute(data, interpreter),
        embive::CBeqz::OPCODE => embive::CBeqz::decode_execute(data, interpreter),
        embive::CBnez::OPCODE => embive::CBnez::decode_execute(data, interpreter),
        embive::CSlli::OPCODE => embive::CSlli::decode_execute(data, interpreter),
        embive::CLwsp::OPCODE => embive::CLwsp::decode_execute(data, interpreter),
        embive::CJrMv::OPCODE => embive::CJrMv::decode_execute(data, interpreter),
        embive::CEbreakJalrAdd::OPCODE => embive::CEbreakJalrAdd::decode_execute(data, interpreter),
        embive::CSwsp::OPCODE => embive::CSwsp::decode_execute(data, interpreter),
        embive::Auipc::OPCODE => embive::Auipc::decode_execute(data, interpreter),
        embive::Branch::OPCODE => embive::Branch::decode_execute(data, interpreter),
        embive::Jal::OPCODE => embive::Jal::decode_execute(data, interpreter),
        embive::Jalr::OPCODE => embive::Jalr::decode_execute(data, interpreter),
        embive::LoadStore::OPCODE => embive::LoadStore::decode_execute(data, interpreter),
        embive::Lui::OPCODE => embive::Lui::decode_execute(data, interpreter),
        embive::OpImm::OPCODE => embive::OpImm::decode_execute(data, interpreter),
        embive::OpAmo::OPCODE => embive::OpAmo::decode_execute(data, interpreter),
        embive::SystemMiscMem::OPCODE => embive::SystemMiscMem::decode_execute(data, interpreter),
        _ => Err(Error::InvalidInstruction(data)),
    }
}
