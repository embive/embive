use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const LB_FUNC: u8 = 0b000;
const LH_FUNC: u8 = 0b001;
const LW_FUNC: u8 = 0b010;
const LBU_FUNC: u8 = 0b100;
const LHU_FUNC: u8 = 0b101;

impl Convert for riscv::Load {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        // Convert funct3
        match inst.func {
            LB_FUNC => inst.func = embive::LoadStore::LB_FUNC,
            LH_FUNC => inst.func = embive::LoadStore::LH_FUNC,
            LW_FUNC => inst.func = embive::LoadStore::LW_FUNC,
            LBU_FUNC => inst.func = embive::LoadStore::LBU_FUNC,
            LHU_FUNC => inst.func = embive::LoadStore::LHU_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::LoadStore, inst))
    }
}
