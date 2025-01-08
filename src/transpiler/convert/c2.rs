use crate::format::{Format, TypeCI4, TypeCI5, TypeCR, TypeCSS};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{c_bit12, c_funct3, embive_raw, Convert, RawInstruction};

const C_SLLI_FUNCT3: u8 = 0b000;
const C_LWSP_FUNCT3: u8 = 0b010;
const C_JR_MV_EBREAK_JALR_ADD_FUNCT3: u8 = 0b100;
const C_SWSP_FUNCT3: u8 = 0b110;

const C_JR_MV_BIT12: u8 = 0b0;

impl Convert for riscv::C2 {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        // Each instruction has a different funct3 value
        match c_funct3(data) {
            C_SLLI_FUNCT3 => {
                let inst = TypeCI4::from_riscv(data);
                Ok(embive_raw!(embive::CSlli, inst))
            }
            C_LWSP_FUNCT3 => {
                let inst = TypeCI5::from_riscv(data);
                Ok(embive_raw!(embive::CLwsp, inst))
            }
            C_JR_MV_EBREAK_JALR_ADD_FUNCT3 => {
                let inst = TypeCR::from_riscv(data);

                // JR, MV, EBREAK, JALR and ADD have the same funct3 value
                if c_bit12(data) == C_JR_MV_BIT12 {
                    Ok(embive_raw!(embive::CJrMv, inst))
                } else {
                    Ok(embive_raw!(embive::CEbreakJalrAdd, inst))
                }
            }
            C_SWSP_FUNCT3 => {
                let inst = TypeCSS::from_riscv(data);
                Ok(embive_raw!(embive::CSwsp, inst))
            }
            _ => Err(Error::InvalidInstruction(data & 0xFFFF)),
        }
    }
}
