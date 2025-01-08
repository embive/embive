use crate::format::{Format, TypeCIW, TypeCL};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{c_funct3, embive_raw, Convert, RawInstruction};

const C_ADDI4SPN_FUNCT3: u8 = 0b000;
const C_LW_FUNCT3: u8 = 0b010;
const C_SW_FUNCT3: u8 = 0b110;

impl Convert for riscv::C0 {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        // Each instruction has a different funct3 value
        match c_funct3(data) {
            C_ADDI4SPN_FUNCT3 => {
                let inst = TypeCIW::from_riscv(data);
                Ok(embive_raw!(embive::CAddi4spn, inst))
            }
            C_LW_FUNCT3 => {
                let inst = TypeCL::from_riscv(data);
                Ok(embive_raw!(embive::CLw, inst))
            }
            C_SW_FUNCT3 => {
                let inst = TypeCL::from_riscv(data);
                Ok(embive_raw!(embive::CSw, inst))
            }
            _ => Err(Error::InvalidInstruction(data & 0xFFFF)),
        }
    }
}
