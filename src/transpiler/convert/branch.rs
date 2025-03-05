use crate::format::{Format, TypeB};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const BEQ_FUNC: u8 = 0b000;
const BNE_FUNC: u8 = 0b001;
const BLT_FUNC: u8 = 0b100;
const BGE_FUNC: u8 = 0b101;
const BLTU_FUNC: u8 = 0b110;
const BGEU_FUNC: u8 = 0b111;

impl Convert for riscv::Branch {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeB::from_riscv(data);

        // Convert funct3
        match inst.func {
            BEQ_FUNC => inst.func = embive::Branch::BEQ_FUNC,
            BNE_FUNC => inst.func = embive::Branch::BNE_FUNC,
            BLT_FUNC => inst.func = embive::Branch::BLT_FUNC,
            BGE_FUNC => inst.func = embive::Branch::BGE_FUNC,
            BLTU_FUNC => inst.func = embive::Branch::BLTU_FUNC,
            BGEU_FUNC => inst.func = embive::Branch::BGEU_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::Branch, inst))
    }
}
