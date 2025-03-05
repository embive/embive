//! Instruction conversion module.
mod amo;
mod auipc;
mod branch;
mod c0;
mod c1;
mod c2;
mod jal;
mod jalr;
mod load;
mod lui;
mod misc_mem;
mod op;
mod op_imm;
mod store;
mod system;

use super::Error;
use crate::format::Size;
use crate::instruction::riscv;

/// Compressed Instruction Funct3
#[inline(always)]
fn c_funct3(data: u32) -> u8 {
    ((data >> 13) & 0b111) as u8
}

/// Compressed Instruction Bit 12
#[inline(always)]
fn c_bit12(data: u32) -> u8 {
    ((data >> 12) & 0b1) as u8
}

/// Compressed Instruction Bits 11-10
#[inline(always)]
fn c_bits11_10(data: u32) -> u8 {
    ((data >> 10) & 0b11) as u8
}

/// Compressed Instruction Bits 6-5
#[inline(always)]
fn c_bits6_5(data: u32) -> u8 {
    ((data >> 5) & 0b11) as u8
}

/// Raw instruction struct.
pub struct RawInstruction {
    pub data: u32,
    pub size: Size,
}

impl RawInstruction {
    /// Create a new raw instruction.
    ///
    /// Arguments:
    /// - `data`: The raw instruction data.
    /// - `size`: The size of the instruction.
    ///
    /// Returns:
    /// - `RawInstruction`: The raw instruction.
    pub fn new(data: u32, size: Size) -> RawInstruction {
        RawInstruction { data, size }
    }
}

/// Convert trait. All instructions must implement this trait.
trait Convert {
    /// Convert the instruction from RISC-V to Embive.
    ///
    /// Arguments:
    /// - `data`: value representing the RISC-V instruction.
    ///
    /// Returns:
    /// - `Ok(u32)`: Instruction converted successfully, returns the raw Embive instruction.
    /// - `Err(Error)`: Failed to convert instruction.
    fn convert(data: u32) -> Result<RawInstruction, Error>;
}

/// Macro for converting a decoded instruction to an encoded Embive instruction.
///
/// Arguments:
/// - `inst`: The instruction type to convert to.
/// - `data`: The decoded instruction.
macro_rules! embive_raw {
    ($inst:ty, $data:expr) => {{
        use crate::instruction::embive::InstructionImpl;
        let inst = <$inst>::from($data);
        RawInstruction::new(inst.encode() | <$inst>::opcode() as u32, <$inst>::size())
    }};
}
use embive_raw;

/// Convert a RISC-V instruction to Embive format.
///
/// # Arguments
/// - `data`: value representing the RISC-V instruction.
///
/// # Returns
/// - `Ok(RawInstruction)`: The raw Embive instruction.
/// - `Err(Error)`: The RISC-V instruction is invalid.
pub fn convert(data: u32) -> Result<RawInstruction, Error> {
    match (data & 0b11) as u8 {
        riscv::C0::OPCODE => riscv::C0::convert(data),
        riscv::C1::OPCODE => riscv::C1::convert(data),
        riscv::C2::OPCODE => riscv::C2::convert(data),
        _ => match (data & 0b111_1111) as u8 {
            riscv::Load::OPCODE => riscv::Load::convert(data),
            riscv::MiscMem::OPCODE => riscv::MiscMem::convert(data),
            riscv::OpImm::OPCODE => riscv::OpImm::convert(data),
            riscv::Auipc::OPCODE => riscv::Auipc::convert(data),
            riscv::Store::OPCODE => riscv::Store::convert(data),
            riscv::Amo::OPCODE => riscv::Amo::convert(data),
            riscv::Op::OPCODE => riscv::Op::convert(data),
            riscv::Lui::OPCODE => riscv::Lui::convert(data),
            riscv::Branch::OPCODE => riscv::Branch::convert(data),
            riscv::Jalr::OPCODE => riscv::Jalr::convert(data),
            riscv::Jal::OPCODE => riscv::Jal::convert(data),
            riscv::System::OPCODE => riscv::System::convert(data),
            _ => Err(Error::InvalidInstruction(data)),
        },
    }
}
