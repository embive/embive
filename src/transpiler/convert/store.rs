use crate::format::{Format, TypeI, TypeS};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const SB_FUNC: u8 = 0b000;
const SH_FUNC: u8 = 0b001;
const SW_FUNC: u8 = 0b010;

impl Convert for riscv::Store {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let inst_s = TypeS::from_riscv(data);

        // Convert to TypeI, as we use the same format as the Load instruction
        let mut inst = TypeI {
            imm: inst_s.imm,
            rs1: inst_s.rs1,
            rd_rs2: inst_s.rs2,
            func: inst_s.func,
        };

        // Convert funct3
        match inst_s.func {
            SB_FUNC => inst.func = embive::LoadStore::SB_FUNC,
            SH_FUNC => inst.func = embive::LoadStore::SH_FUNC,
            SW_FUNC => inst.func = embive::LoadStore::SW_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::LoadStore, inst))
    }
}
