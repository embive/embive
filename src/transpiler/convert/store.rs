use crate::format::{Format, TypeI, TypeS};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const SB_FUNCT3: u8 = 0b000;
const SH_FUNCT3: u8 = 0b001;
const SW_FUNCT3: u8 = 0b010;

impl Convert for riscv::Store {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let inst_s = TypeS::from_riscv(data);

        // Convert to TypeI, as we use the same format as the Load instruction
        let mut inst = TypeI {
            imm: inst_s.imm,
            rs1: inst_s.rs1,
            rd_rs2: inst_s.rs2,
            funct3: inst_s.funct3,
        };

        // Convert funct3
        match inst_s.funct3 {
            SB_FUNCT3 => inst.funct3 = embive::LoadStore::SB_FUNCT3,
            SH_FUNCT3 => inst.funct3 = embive::LoadStore::SH_FUNCT3,
            SW_FUNCT3 => inst.funct3 = embive::LoadStore::SW_FUNCT3,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::LoadStore, inst))
    }
}
