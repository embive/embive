use crate::format::{Format, TypeJ};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

impl Convert for riscv::Jal {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let inst = TypeJ::from_riscv(data);

        Ok(embive_raw!(embive::Jal, inst))
    }
}
