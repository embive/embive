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
        if (inst.funct10 & 0b111) as u8 != WORD_WIDTH {
            return Err(Error::InvalidInstruction(data));
        }

        // Convert the funct5 field to OpAmo funct10
        match (inst.funct10 >> 5) as u8 {
            LR_FUNCT5 => inst.funct10 = embive::OpAmo::LR_FUNCT10,
            SC_FUNCT5 => inst.funct10 = embive::OpAmo::SC_FUNCT10,
            AMOSWAP_FUNCT5 => inst.funct10 = embive::OpAmo::AMOSWAP_FUNCT10,
            AMOADD_FUNCT5 => inst.funct10 = embive::OpAmo::AMOADD_FUNCT10,
            AMOXOR_FUNCT5 => inst.funct10 = embive::OpAmo::AMOXOR_FUNCT10,
            AMOAND_FUNCT5 => inst.funct10 = embive::OpAmo::AMOAND_FUNCT10,
            AMOOR_FUNCT5 => inst.funct10 = embive::OpAmo::AMOOR_FUNCT10,
            AMOMIN_FUNCT5 => inst.funct10 = embive::OpAmo::AMOMIN_FUNCT10,
            AMOMAX_FUNCT5 => inst.funct10 = embive::OpAmo::AMOMAX_FUNCT10,
            AMOMINU_FUNCT5 => inst.funct10 = embive::OpAmo::AMOMINU_FUNCT10,
            AMOMAXU_FUNCT5 => inst.funct10 = embive::OpAmo::AMOMAXU_FUNCT10,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpAmo, inst))
    }
}
