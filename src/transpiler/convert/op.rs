use crate::format::{Format, TypeR};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const MUL_ADD_SUB_FUNC: u8 = 0b000;
const DIV_XOR_FUNC: u8 = 0b100;
const REM_OR_FUNC: u8 = 0b110;
const REMU_AND_FUNC: u8 = 0b111;
const MULH_SLL_FUNC: u8 = 0b001;
const DIVU_SRL_SRA_FUNC: u8 = 0b101;
const MULHSU_SLT_FUNC: u8 = 0b010;
const MULHU_SLTU_FUNC: u8 = 0b011;

const M_EXT_FUNCT7: u8 = 0b0000001;
const SUB_SRA_FUNCT7: u8 = 0b0100000;

const ADD_FUNC: u16 = MUL_ADD_SUB_FUNC as u16;
const SUB_FUNC: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNC as u16;
const XOR_FUNC: u16 = DIV_XOR_FUNC as u16;
const OR_FUNC: u16 = REM_OR_FUNC as u16;
const AND_FUNC: u16 = REMU_AND_FUNC as u16;
const SLL_FUNC: u16 = MULH_SLL_FUNC as u16;
const SRL_FUNC: u16 = DIVU_SRL_SRA_FUNC as u16;
const SRA_FUNC: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNC as u16;
const SLT_FUNC: u16 = MULHSU_SLT_FUNC as u16;
const SLTU_FUNC: u16 = MULHU_SLTU_FUNC as u16;

const MUL_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNC as u16;
const DIV_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIV_XOR_FUNC as u16;
const REM_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REM_OR_FUNC as u16;
const REMU_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REMU_AND_FUNC as u16;
const MULH_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULH_SLL_FUNC as u16;
const DIVU_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNC as u16;
const MULHSU_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHSU_SLT_FUNC as u16;
const MULHU_FUNC: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHU_SLTU_FUNC as u16;

impl Convert for riscv::Op {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeR::from_riscv(data);

        // Convert funct10
        match inst.func {
            ADD_FUNC => inst.func = embive::OpAmo::ADD_FUNC,
            SUB_FUNC => inst.func = embive::OpAmo::SUB_FUNC,
            SLL_FUNC => inst.func = embive::OpAmo::SLL_FUNC,
            SLT_FUNC => inst.func = embive::OpAmo::SLT_FUNC,
            SLTU_FUNC => inst.func = embive::OpAmo::SLTU_FUNC,
            XOR_FUNC => inst.func = embive::OpAmo::XOR_FUNC,
            SRL_FUNC => inst.func = embive::OpAmo::SRL_FUNC,
            SRA_FUNC => inst.func = embive::OpAmo::SRA_FUNC,
            OR_FUNC => inst.func = embive::OpAmo::OR_FUNC,
            AND_FUNC => inst.func = embive::OpAmo::AND_FUNC,
            MUL_FUNC => inst.func = embive::OpAmo::MUL_FUNC,
            MULH_FUNC => inst.func = embive::OpAmo::MULH_FUNC,
            MULHSU_FUNC => inst.func = embive::OpAmo::MULHSU_FUNC,
            MULHU_FUNC => inst.func = embive::OpAmo::MULHU_FUNC,
            DIV_FUNC => inst.func = embive::OpAmo::DIV_FUNC,
            DIVU_FUNC => inst.func = embive::OpAmo::DIVU_FUNC,
            REM_FUNC => inst.func = embive::OpAmo::REM_FUNC,
            REMU_FUNC => inst.func = embive::OpAmo::REMU_FUNC,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpAmo, inst))
    }
}
