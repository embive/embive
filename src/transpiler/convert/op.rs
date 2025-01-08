use crate::format::{Format, TypeR};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

const MUL_ADD_SUB_FUNCT3: u8 = 0b000;
const DIV_XOR_FUNCT3: u8 = 0b100;
const REM_OR_FUNCT3: u8 = 0b110;
const REMU_AND_FUNCT3: u8 = 0b111;
const MULH_SLL_FUNCT3: u8 = 0b001;
const DIVU_SRL_SRA_FUNCT3: u8 = 0b101;
const MULHSU_SLT_FUNCT3: u8 = 0b010;
const MULHU_SLTU_FUNCT3: u8 = 0b011;

const M_EXT_FUNCT7: u8 = 0b0000001;
const SUB_SRA_FUNCT7: u8 = 0b0100000;

const ADD_FUNCT10: u16 = MUL_ADD_SUB_FUNCT3 as u16;
const SUB_FUNCT10: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNCT3 as u16;
const XOR_FUNCT10: u16 = DIV_XOR_FUNCT3 as u16;
const OR_FUNCT10: u16 = REM_OR_FUNCT3 as u16;
const AND_FUNCT10: u16 = REMU_AND_FUNCT3 as u16;
const SLL_FUNCT10: u16 = MULH_SLL_FUNCT3 as u16;
const SRL_FUNCT10: u16 = DIVU_SRL_SRA_FUNCT3 as u16;
const SRA_FUNCT10: u16 = ((SUB_SRA_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNCT3 as u16;
const SLT_FUNCT10: u16 = MULHSU_SLT_FUNCT3 as u16;
const SLTU_FUNCT10: u16 = MULHU_SLTU_FUNCT3 as u16;

const MUL_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MUL_ADD_SUB_FUNCT3 as u16;
const DIV_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIV_XOR_FUNCT3 as u16;
const REM_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REM_OR_FUNCT3 as u16;
const REMU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | REMU_AND_FUNCT3 as u16;
const MULH_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULH_SLL_FUNCT3 as u16;
const DIVU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | DIVU_SRL_SRA_FUNCT3 as u16;
const MULHSU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHSU_SLT_FUNCT3 as u16;
const MULHU_FUNCT10: u16 = ((M_EXT_FUNCT7 as u16) << 3) | MULHU_SLTU_FUNCT3 as u16;

impl Convert for riscv::Op {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeR::from_riscv(data);

        // Convert funct10
        match inst.funct10 {
            ADD_FUNCT10 => inst.funct10 = embive::OpAmo::ADD_FUNCT10,
            SUB_FUNCT10 => inst.funct10 = embive::OpAmo::SUB_FUNCT10,
            SLL_FUNCT10 => inst.funct10 = embive::OpAmo::SLL_FUNCT10,
            SLT_FUNCT10 => inst.funct10 = embive::OpAmo::SLT_FUNCT10,
            SLTU_FUNCT10 => inst.funct10 = embive::OpAmo::SLTU_FUNCT10,
            XOR_FUNCT10 => inst.funct10 = embive::OpAmo::XOR_FUNCT10,
            SRL_FUNCT10 => inst.funct10 = embive::OpAmo::SRL_FUNCT10,
            SRA_FUNCT10 => inst.funct10 = embive::OpAmo::SRA_FUNCT10,
            OR_FUNCT10 => inst.funct10 = embive::OpAmo::OR_FUNCT10,
            AND_FUNCT10 => inst.funct10 = embive::OpAmo::AND_FUNCT10,
            MUL_FUNCT10 => inst.funct10 = embive::OpAmo::MUL_FUNCT10,
            MULH_FUNCT10 => inst.funct10 = embive::OpAmo::MULH_FUNCT10,
            MULHSU_FUNCT10 => inst.funct10 = embive::OpAmo::MULHSU_FUNCT10,
            MULHU_FUNCT10 => inst.funct10 = embive::OpAmo::MULHU_FUNCT10,
            DIV_FUNCT10 => inst.funct10 = embive::OpAmo::DIV_FUNCT10,
            DIVU_FUNCT10 => inst.funct10 = embive::OpAmo::DIVU_FUNCT10,
            REM_FUNCT10 => inst.funct10 = embive::OpAmo::REM_FUNCT10,
            REMU_FUNCT10 => inst.funct10 = embive::OpAmo::REMU_FUNCT10,
            _ => return Err(Error::InvalidInstruction(data)),
        }

        Ok(embive_raw!(embive::OpAmo, inst))
    }
}
