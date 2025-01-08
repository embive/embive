use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const ADDI_FUNC3: u8 = 0b000;
const SLLI_FUNC3: u8 = 0b001;
const SLTI_FUNC3: u8 = 0b010;
const SLTIU_FUNC3: u8 = 0b011;
const XORI_FUNC3: u8 = 0b100;
const SRLI_SRAI_FUNC3: u8 = 0b101;
const ORI_FUNC3: u8 = 0b110;
const ANDI_FUNC3: u8 = 0b111;

impl Convert for riscv::OpImm {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        // Convert funct3
        match inst.funct3 {
            ADDI_FUNC3 => inst.funct3 = embive::OpImm::ADDI_FUNC3,
            SLLI_FUNC3 => inst.funct3 = embive::OpImm::SLLI_FUNC3,
            SLTI_FUNC3 => inst.funct3 = embive::OpImm::SLTI_FUNC3,
            SLTIU_FUNC3 => inst.funct3 = embive::OpImm::SLTIU_FUNC3,
            XORI_FUNC3 => inst.funct3 = embive::OpImm::XORI_FUNC3,
            SRLI_SRAI_FUNC3 => inst.funct3 = embive::OpImm::SRLI_SRAI_FUNC3,
            ORI_FUNC3 => inst.funct3 = embive::OpImm::ORI_FUNC3,
            ANDI_FUNC3 => inst.funct3 = embive::OpImm::ANDI_FUNC3,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpImm, inst))
    }
}
