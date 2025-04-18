use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

impl Convert for riscv::Jalr {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let inst = TypeI::from_riscv(data);

        Ok(embive_raw!(embive::Jalr, inst))
    }
}
