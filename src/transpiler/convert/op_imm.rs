use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const ADDI_FUNC: u8 = 0b000;
const SLLI_FUNC: u8 = 0b001;
const SLTI_FUNC: u8 = 0b010;
const SLTIU_FUNC: u8 = 0b011;
const XORI_FUNC: u8 = 0b100;
const SRLI_SRAI_FUNC: u8 = 0b101;
const ORI_FUNC: u8 = 0b110;
const ANDI_FUNC: u8 = 0b111;

impl Convert for riscv::OpImm {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        // Convert funct3
        match inst.func {
            ADDI_FUNC => inst.func = embive::OpImm::ADDI_FUNC,
            SLLI_FUNC => inst.func = embive::OpImm::SLLI_FUNC,
            SLTI_FUNC => inst.func = embive::OpImm::SLTI_FUNC,
            SLTIU_FUNC => inst.func = embive::OpImm::SLTIU_FUNC,
            XORI_FUNC => inst.func = embive::OpImm::XORI_FUNC,
            SRLI_SRAI_FUNC => inst.func = embive::OpImm::SRLI_SRAI_FUNC,
            ORI_FUNC => inst.func = embive::OpImm::ORI_FUNC,
            ANDI_FUNC => inst.func = embive::OpImm::ANDI_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpImm, inst))
    }
}
