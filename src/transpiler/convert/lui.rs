use crate::format::{Format, TypeU};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

impl Convert for riscv::Lui {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let inst = TypeU::from_riscv(data);

        Ok(embive_raw!(embive::Lui, inst))
    }
}
