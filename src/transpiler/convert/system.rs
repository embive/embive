use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

pub const ECALL_IMM: i32 = 0b0;
pub const EBREAK_IMM: i32 = 0b1;
pub const WFI_IMM: i32 = 0b1_0000_0101;
pub const MRET_IMM: i32 = 0b11_0000_0010;

pub const CSRRW_FUNCT3: u8 = 0b001;
pub const CSRRS_FUNCT3: u8 = 0b010;
pub const CSRRC_FUNCT3: u8 = 0b011;
pub const CSRRWI_FUNCT3: u8 = 0b101;
pub const CSRRSI_FUNCT3: u8 = 0b110;
pub const CSRRCI_FUNCT3: u8 = 0b111;

impl Convert for riscv::System {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        if inst.funct3 == embive::SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3 {
            // Convert immediates
            match inst.imm {
                ECALL_IMM => inst.imm = embive::SystemMiscMem::ECALL_IMM,
                EBREAK_IMM => inst.imm = embive::SystemMiscMem::EBREAK_IMM,
                WFI_IMM => inst.imm = embive::SystemMiscMem::WFI_IMM,
                MRET_IMM => inst.imm = embive::SystemMiscMem::MRET_IMM,
                _ => {}
            }
        } else {
            // Convert funct3
            match inst.funct3 {
                CSRRW_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRW_FUNCT3,
                CSRRS_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRS_FUNCT3,
                CSRRC_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRC_FUNCT3,
                CSRRWI_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRWI_FUNCT3,
                CSRRSI_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRSI_FUNCT3,
                CSRRCI_FUNCT3 => inst.funct3 = embive::SystemMiscMem::CSRRCI_FUNCT3,
                _ => return Err(Error::InvalidInstruction(data)),
            }
        }

        Ok(embive_raw!(embive::SystemMiscMem, inst))
    }
}
