use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const LB_FUNCT3: u8 = 0b000;
const LH_FUNCT3: u8 = 0b001;
const LW_FUNCT3: u8 = 0b010;
const LBU_FUNCT3: u8 = 0b100;
const LHU_FUNCT3: u8 = 0b101;

impl Convert for riscv::Load {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        // Convert funct3
        match inst.funct3 {
            LB_FUNCT3 => inst.funct3 = embive::LoadStore::LB_FUNCT3,
            LH_FUNCT3 => inst.funct3 = embive::LoadStore::LH_FUNCT3,
            LW_FUNCT3 => inst.funct3 = embive::LoadStore::LW_FUNCT3,
            LBU_FUNCT3 => inst.funct3 = embive::LoadStore::LBU_FUNCT3,
            LHU_FUNCT3 => inst.funct3 = embive::LoadStore::LHU_FUNCT3,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::LoadStore, inst))
    }
}
