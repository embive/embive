use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

impl Convert for riscv::MiscMem {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);
        inst.funct3 = embive::SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3;
        inst.imm = embive::SystemMiscMem::FENCEI_IMM;

        Ok(embive_raw!(embive::SystemMiscMem, inst))
    }
}
