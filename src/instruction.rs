//! Instruction module.

#[cfg(any(feature = "transpiler", feature = "interpreter"))]
mod embive_macro;
#[cfg(feature = "transpiler")]
mod riscv_macro;

#[cfg(any(feature = "transpiler", feature = "interpreter"))]
#[doc(inline)]
pub use embive::Instruction;

/// Embive Instruction
#[cfg(any(feature = "transpiler", feature = "interpreter"))]
pub(crate) mod embive {
    use super::embive_macro::instructions;
    use crate::format::{
        Format, Size, TypeB, TypeCB1, TypeCB2, TypeCB4, TypeCI1, TypeCI2, TypeCI3, TypeCI4,
        TypeCI5, TypeCIW, TypeCJ, TypeCL, TypeCR, TypeCS, TypeCSS, TypeI, TypeJ, TypeR, TypeU,
    };

    /// Embive Instruction Struct
    ///
    /// This struct wraps a raw embive instruction (u32)
    /// with a custom implementation for the Debug trait.
    #[derive(Clone, Copy, PartialEq)]
    pub struct Instruction(u32);

    impl From<u32> for Instruction {
        #[inline(always)]
        fn from(inst: u32) -> Self {
            Self(inst)
        }
    }

    impl From<Instruction> for u32 {
        #[inline(always)]
        fn from(inst: Instruction) -> u32 {
            inst.0
        }
    }

    impl core::fmt::Debug for Instruction {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match decode_instruction!(self.0, fmt, (f)) {
                Some(_) => Ok(()),
                None => write!(f, "Invalid Instruction"),
            }
        }
    }

    /// Embive Instruction Trait
    #[allow(dead_code)]
    pub trait InstructionImpl {
        /// Instruction Opcode
        fn opcode() -> u8;

        /// Instruction size in bytes
        fn size() -> Size;

        /// Encode instruction to u32 (Embive Format)
        fn encode(&self) -> u32;

        /// Decode instruction from u32 (Embive Format)
        fn decode(inst: u32) -> Self;
    }

    // Name, Opcode, Size, Format, Custom Data
    instructions! {
        0 => CAddi4spn: TypeCIW = {};
        1 => CLw: TypeCL = {};
        2 => CSw: TypeCL = {};
        3 => CAddi: TypeCI1 = {};
        4 => CJal: TypeCJ = {};
        5 => CLi: TypeCI1 = {};
        6 => CAddi16sp: TypeCI2 = {};
        7 => CLui: TypeCI3 = {};
        8 => CSrli: TypeCB1 = {};
        9 => CSrai: TypeCB1 = {};
        10 => CAndi: TypeCB2 = {};
        11 => CSub: TypeCS = {};
        12 => CXor: TypeCS = {};
        13 => COr: TypeCS = {};
        14 => CAnd: TypeCS = {};
        15 => CJ: TypeCJ = {};
        16 => CBeqz: TypeCB4 = {};
        17 => CBnez: TypeCB4 = {};
        18 => CSlli: TypeCI4 = {};
        19 => CLwsp: TypeCI5 = {};
        20 => CJrMv: TypeCR = {};
        21 => CEbreakJalrAdd: TypeCR = {};
        22 => CSwsp: TypeCSS = {};
        23 => Auipc: TypeU = {};
        24 => Branch: TypeB = {
            u8: {
                BEQ_FUNC = 0;
                BNE_FUNC = 1;
                BLT_FUNC = 2;
                BGE_FUNC = 3;
                BLTU_FUNC = 4;
                BGEU_FUNC = 5;
            }
        };
        25 => Jal: TypeJ = {};
        26 => Jalr: TypeI = {};
        27 => LoadStore: TypeI = {
            u8: {
                LB_FUNC = 0;
                LH_FUNC = 1;
                LW_FUNC = 2;
                LBU_FUNC = 3;
                LHU_FUNC = 4;
                SB_FUNC = 5;
                SH_FUNC = 6;
                SW_FUNC = 7;
            }
        };
        28 => Lui: TypeU = {};
        29 => OpImm: TypeI = {
            u8: {
                ADDI_FUNC = 0;
                SLLI_FUNC = 1;
                SLTI_FUNC = 2;
                SLTIU_FUNC = 3;
                XORI_FUNC = 4;
                SRLI_SRAI_FUNC = 5;
                ORI_FUNC = 6;
                ANDI_FUNC = 7;
            }
        };
        30 => OpAmo: TypeR = {
            u16: {
                ADD_FUNC = 0;
                SUB_FUNC = 1;
                SLL_FUNC = 2;
                SLT_FUNC = 3;
                SLTU_FUNC = 4;
                XOR_FUNC = 5;
                SRL_FUNC = 6;
                SRA_FUNC = 7;
                OR_FUNC = 8;
                AND_FUNC = 9;
                MUL_FUNC = 10;
                MULH_FUNC = 11;
                MULHSU_FUNC = 12;
                MULHU_FUNC = 13;
                DIV_FUNC = 14;
                DIVU_FUNC = 15;
                REM_FUNC = 16;
                REMU_FUNC = 17;
                LR_FUNC = 18;
                SC_FUNC = 19;
                AMOSWAP_FUNC = 20;
                AMOADD_FUNC = 21;
                AMOXOR_FUNC = 22;
                AMOAND_FUNC = 23;
                AMOOR_FUNC = 24;
                AMOMIN_FUNC = 25;
                AMOMAX_FUNC = 26;
                AMOMINU_FUNC = 27;
                AMOMAXU_FUNC = 28;
            }
        };
        31 => SystemMiscMem: TypeI = {
            i32: {
                ECALL_IMM = 0;
                EBREAK_IMM = 1;
                FENCEI_IMM = 2;
                WFI_IMM = 3;
                MRET_IMM = 4;
            },
            u8: {
                MISC_FUNC = 0;
                CSRRW_FUNC = 1;
                CSRRS_FUNC = 2;
                CSRRC_FUNC = 3;
                CSRRWI_FUNC = 4;
                CSRRSI_FUNC = 5;
                CSRRCI_FUNC = 6;
            }
        };
    }
}

/// RISC-V Instruction
#[cfg(feature = "transpiler")]
pub(crate) mod riscv {
    use super::riscv_macro::instruction;

    // Name, Opcode
    instruction!(C0, 0b00);
    instruction!(C1, 0b01);
    instruction!(C2, 0b10);
    instruction!(Auipc, 0b001_0111);
    instruction!(Amo, 0b010_1111);
    instruction!(Branch, 0b110_0011);
    instruction!(Jal, 0b110_1111);
    instruction!(Jalr, 0b110_0111);
    instruction!(Load, 0b000_0011);
    instruction!(Lui, 0b011_0111);
    instruction!(MiscMem, 0b000_1111);
    instruction!(OpImm, 0b001_0011);
    instruction!(Op, 0b011_0011);
    instruction!(Store, 0b010_0011);
    instruction!(System, 0b111_0011);
}
