use crate::format::{Format, TypeB};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const BEQ_FUNCT3: u8 = 0b000;
const BNE_FUNCT3: u8 = 0b001;
const BLT_FUNCT3: u8 = 0b100;
const BGE_FUNCT3: u8 = 0b101;
const BLTU_FUNCT3: u8 = 0b110;
const BGEU_FUNCT3: u8 = 0b111;

impl Convert for riscv::Branch {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeB::from_riscv(data);

        // Convert funct3
        match inst.funct3 {
            BEQ_FUNCT3 => inst.funct3 = embive::Branch::BEQ_FUNCT3,
            BNE_FUNCT3 => inst.funct3 = embive::Branch::BNE_FUNCT3,
            BLT_FUNCT3 => inst.funct3 = embive::Branch::BLT_FUNCT3,
            BGE_FUNCT3 => inst.funct3 = embive::Branch::BGE_FUNCT3,
            BLTU_FUNCT3 => inst.funct3 = embive::Branch::BLTU_FUNCT3,
            BGEU_FUNCT3 => inst.funct3 = embive::Branch::BGEU_FUNCT3,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::Branch, inst))
    }
}
