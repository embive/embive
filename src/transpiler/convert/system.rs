use crate::format::{Format, TypeI};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{embive_raw, Convert, RawInstruction};

pub const ECALL_IMM: i32 = 0b0;
pub const EBREAK_IMM: i32 = 0b1;
pub const WFI_IMM: i32 = 0b1_0000_0101;
pub const MRET_IMM: i32 = 0b11_0000_0010;

pub const CSRRW_FUNC: u8 = 0b001;
pub const CSRRS_FUNC: u8 = 0b010;
pub const CSRRC_FUNC: u8 = 0b011;
pub const CSRRWI_FUNC: u8 = 0b101;
pub const CSRRSI_FUNC: u8 = 0b110;
pub const CSRRCI_FUNC: u8 = 0b111;

impl Convert for riscv::System {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        let mut inst = TypeI::from_riscv(data);

        if inst.func == embive::SystemMiscMem::MISC_FUNC {
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
            match inst.func {
                CSRRW_FUNC => inst.func = embive::SystemMiscMem::CSRRW_FUNC,
                CSRRS_FUNC => inst.func = embive::SystemMiscMem::CSRRS_FUNC,
                CSRRC_FUNC => inst.func = embive::SystemMiscMem::CSRRC_FUNC,
                CSRRWI_FUNC => inst.func = embive::SystemMiscMem::CSRRWI_FUNC,
                CSRRSI_FUNC => inst.func = embive::SystemMiscMem::CSRRSI_FUNC,
                CSRRCI_FUNC => inst.func = embive::SystemMiscMem::CSRRCI_FUNC,
                _ => return Err(Error::InvalidInstruction(data)),
            }
        }

        Ok(embive_raw!(embive::SystemMiscMem, inst))
    }
}
