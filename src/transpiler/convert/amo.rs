use crate::format::{Format, TypeR};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const WORD_WIDTH: u8 = 0b010;

const LR_FUNCT5: u8 = 0b00010;
const SC_FUNCT5: u8 = 0b00011;
const AMOSWAP_FUNCT5: u8 = 0b00001;
const AMOADD_FUNCT5: u8 = 0b00000;
const AMOXOR_FUNCT5: u8 = 0b00100;
const AMOAND_FUNCT5: u8 = 0b01100;
const AMOOR_FUNCT5: u8 = 0b01000;
const AMOMIN_FUNCT5: u8 = 0b10000;
const AMOMAX_FUNCT5: u8 = 0b10100;
const AMOMINU_FUNCT5: u8 = 0b11000;
const AMOMAXU_FUNCT5: u8 = 0b11100;

impl Convert for riscv::Amo {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeR::from_riscv(data);

        // Check word width is 32 bits
        if (inst.func & 0b111) as u8 != WORD_WIDTH {
            return Err(Error::InvalidInstruction(data));
        }

        // Convert the funct5 field to OpAmo funct10
        match (inst.func >> 5) as u8 {
            LR_FUNCT5 => inst.func = embive::OpAmo::LR_FUNC,
            SC_FUNCT5 => inst.func = embive::OpAmo::SC_FUNC,
            AMOSWAP_FUNCT5 => inst.func = embive::OpAmo::AMOSWAP_FUNC,
            AMOADD_FUNCT5 => inst.func = embive::OpAmo::AMOADD_FUNC,
            AMOXOR_FUNCT5 => inst.func = embive::OpAmo::AMOXOR_FUNC,
            AMOAND_FUNCT5 => inst.func = embive::OpAmo::AMOAND_FUNC,
            AMOOR_FUNCT5 => inst.func = embive::OpAmo::AMOOR_FUNC,
            AMOMIN_FUNCT5 => inst.func = embive::OpAmo::AMOMIN_FUNC,
            AMOMAX_FUNCT5 => inst.func = embive::OpAmo::AMOMAX_FUNC,
            AMOMINU_FUNCT5 => inst.func = embive::OpAmo::AMOMINU_FUNC,
            AMOMAXU_FUNCT5 => inst.func = embive::OpAmo::AMOMAXU_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpAmo, inst))
    }
}
