use crate::format::{Format, TypeCB1, TypeCB2, TypeCB4, TypeCI1, TypeCI2, TypeCI3, TypeCJ, TypeCS};
use crate::instruction::{embive, riscv};
use crate::transpiler::Error;

use super::{c_bit12, c_bits11_10, c_bits6_5, c_funct3, embive_raw, Convert, RawInstruction};

const C_ADDI_FUNCT3: u8 = 0b000;
const C_JAL_FUNCT3: u8 = 0b001;
const C_LI_FUNCT3: u8 = 0b010;
const C_ADDI16SP_LUI_FUNCT3: u8 = 0b011;
const C_SRLI_SRAI_ANDI_SUB_XOR_OR_AND_FUNC3: u8 = 0b100;
const C_J_FUNCT3: u8 = 0b101;
const C_BEQZ_FUNCT3: u8 = 0b110;
const C_BNEZ_FUNCT3: u8 = 0b111;

const C_SRLI_BITS11_10: u8 = 0b00;
const C_SRAI_BITS11_10: u8 = 0b01;
const C_ANDI_BITS11_10: u8 = 0b10;

const C_SUB_XOR_OR_AND_BIT12: u8 = 0b0;
const C_SUB_XOR_OR_AND_BITS11_10: u8 = 0b11;
const C_SUB_BITS6_5: u8 = 0b00;
const C_XOR_BITS6_5: u8 = 0b01;
const C_OR_BITS6_5: u8 = 0b10;
const C_AND_BITS6_5: u8 = 0b11;

impl Convert for riscv::C1 {
    fn convert(data: u32) -> Result<RawInstruction, Error> {
        // Each instruction has a different funct3
        match c_funct3(data) {
            C_ADDI_FUNCT3 => {
                let inst = TypeCI1::from_riscv(data);
                Ok(embive_raw!(embive::CAddi, inst))
            }
            C_JAL_FUNCT3 => {
                let inst = TypeCJ::from_riscv(data);
                Ok(embive_raw!(embive::CJal, inst))
            }
            C_LI_FUNCT3 => {
                let inst = TypeCI1::from_riscv(data);
                Ok(embive_raw!(embive::CLi, inst))
            }
            C_ADDI16SP_LUI_FUNCT3 => {
                // C_ADDI16SP and C_LUI share the same funct3
                let inst = TypeCI3::from_riscv(data);

                // C.ADDI16SP has rd = 2
                if inst.rd_rs1 == 2 {
                    // C.ADDI16SP immediate value is different from C.LUI
                    let inst = TypeCI2::from_riscv(data);
                    Ok(embive_raw!(embive::CAddi16sp, inst))
                } else {
                    Ok(embive_raw!(embive::CLui, inst))
                }
            }
            C_SRLI_SRAI_ANDI_SUB_XOR_OR_AND_FUNC3 => match c_bits11_10(data) {
                // C_SRLI, C_SRAI, C_ANDI, C_SUB, C_XOR, C_OR and C_AND share the same funct3
                C_SRLI_BITS11_10 => {
                    let inst = TypeCB1::from_riscv(data);
                    Ok(embive_raw!(embive::CSrli, inst))
                }
                C_SRAI_BITS11_10 => {
                    let inst = TypeCB1::from_riscv(data);
                    Ok(embive_raw!(embive::CSrai, inst))
                }
                C_ANDI_BITS11_10 => {
                    let inst = TypeCB2::from_riscv(data);
                    Ok(embive_raw!(embive::CAndi, inst))
                }
                C_SUB_XOR_OR_AND_BITS11_10 if (c_bit12(data) == C_SUB_XOR_OR_AND_BIT12) => {
                    // C_SUB, C_XOR, C_OR and C_AND share the same bit 11 and 10
                    let inst = TypeCS::from_riscv(data);
                    match c_bits6_5(data) {
                        C_SUB_BITS6_5 => Ok(embive_raw!(embive::CSub, inst)),
                        C_XOR_BITS6_5 => Ok(embive_raw!(embive::CXor, inst)),
                        C_OR_BITS6_5 => Ok(embive_raw!(embive::COr, inst)),
                        C_AND_BITS6_5 => Ok(embive_raw!(embive::CAnd, inst)),
                        _ => Err(Error::InvalidInstruction(data & 0xFFFF)),
                    }
                }
                _ => Err(Error::InvalidInstruction(data & 0xFFFF)),
            },
            C_J_FUNCT3 => {
                let inst = TypeCJ::from_riscv(data);
                Ok(embive_raw!(embive::CJ, inst))
            }
            C_BEQZ_FUNCT3 => {
                let inst = TypeCB4::from_riscv(data);
                Ok(embive_raw!(embive::CBeqz, inst))
            }
            C_BNEZ_FUNCT3 => {
                let inst = TypeCB4::from_riscv(data);
                Ok(embive_raw!(embive::CBnez, inst))
            }
            _ => Err(Error::InvalidInstruction(data & 0xFFFF)),
        }
    }
}
