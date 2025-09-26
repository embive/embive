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

use crate::instruction::Instruction;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use crate::instruction::embive::decode_instruction;

/// Execute trait. All instructions must implement this trait.
trait Execute<M: Memory> {
    /// Execute the instruction.
    ///
    /// Arguments:
    /// - `interpreter`: Mutable pointer to embive interpreter.
    ///
    /// Returns:
    /// - `Ok(EngineState)`: Instruction executed successfully.
    /// - `Err(Error)`: Failed to execute instruction.
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error>;
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
    data: Instruction,
) -> Result<State, Error> {
    match decode_instruction!(data, execute, (interpreter)) {
        Some(state) => state,
        None => Err(Error::InvalidInstruction(interpreter.program_counter)),
    }
}
