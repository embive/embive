//! Instruction module.
#[cfg(any(feature = "transpiler", feature = "interpreter"))]
mod embive_macro;
#[cfg(feature = "transpiler")]
mod riscv_macro;

/// Instruction Size
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Size {
    Half = 2,
    Word = 4,
}

/// Embive Instruction
#[cfg(any(feature = "transpiler", feature = "interpreter"))]
pub mod embive {
    use super::{embive_macro::instruction, Size};
    use crate::format::{
        Format, TypeB, TypeCB1, TypeCB2, TypeCB4, TypeCI1, TypeCI2, TypeCI3, TypeCI4, TypeCI5,
        TypeCIW, TypeCJ, TypeCL, TypeCR, TypeCS, TypeCSS, TypeI, TypeJ, TypeR, TypeU,
    };

    // Name, Opcode, Size, Format, Custom (code/data)
    instruction!(CAddi4spn, 0, Size::Half, TypeCIW, {});
    instruction!(CLw, 1, Size::Half, TypeCL, {});
    instruction!(CSw, 2, Size::Half, TypeCL, {});
    instruction!(CAddi, 3, Size::Half, TypeCI1, {});
    instruction!(CJal, 4, Size::Half, TypeCJ, {});
    instruction!(CLi, 5, Size::Half, TypeCI1, {});
    instruction!(CAddi16sp, 6, Size::Half, TypeCI2, {});
    instruction!(CLui, 7, Size::Half, TypeCI3, {});
    instruction!(CSrli, 8, Size::Half, TypeCB1, {});
    instruction!(CSrai, 9, Size::Half, TypeCB1, {});
    instruction!(CAndi, 10, Size::Half, TypeCB2, {});
    instruction!(CSub, 11, Size::Half, TypeCS, {});
    instruction!(CXor, 12, Size::Half, TypeCS, {});
    instruction!(COr, 13, Size::Half, TypeCS, {});
    instruction!(CAnd, 14, Size::Half, TypeCS, {});
    instruction!(CJ, 15, Size::Half, TypeCJ, {});
    instruction!(CBeqz, 16, Size::Half, TypeCB4, {});
    instruction!(CBnez, 17, Size::Half, TypeCB4, {});
    instruction!(CSlli, 18, Size::Half, TypeCI4, {});
    instruction!(CLwsp, 19, Size::Half, TypeCI5, {});
    instruction!(CJrMv, 20, Size::Half, TypeCR, {});
    instruction!(CEbreakJalrAdd, 21, Size::Half, TypeCR, {});
    instruction!(CSwsp, 22, Size::Half, TypeCSS, {});
    instruction!(Auipc, 23, Size::Word, TypeU, {});
    instruction!(Branch, 24, Size::Word, TypeB, {
        pub const BEQ_FUNCT3: u8 = 0;
        pub const BNE_FUNCT3: u8 = 1;
        pub const BLT_FUNCT3: u8 = 2;
        pub const BGE_FUNCT3: u8 = 3;
        pub const BLTU_FUNCT3: u8 = 4;
        pub const BGEU_FUNCT3: u8 = 5;
    });
    instruction!(Jal, 25, Size::Word, TypeJ, {});
    instruction!(Jalr, 26, Size::Word, TypeI, {});
    instruction!(LoadStore, 27, Size::Word, TypeI, {
        pub const LB_FUNCT3: u8 = 0;
        pub const LH_FUNCT3: u8 = 1;
        pub const LW_FUNCT3: u8 = 2;
        pub const LBU_FUNCT3: u8 = 3;
        pub const LHU_FUNCT3: u8 = 4;
        pub const SB_FUNCT3: u8 = 5;
        pub const SH_FUNCT3: u8 = 6;
        pub const SW_FUNCT3: u8 = 7;
    });
    instruction!(Lui, 28, Size::Word, TypeU, {});
    instruction!(OpImm, 29, Size::Word, TypeI, {
        pub const ADDI_FUNC3: u8 = 0;
        pub const SLLI_FUNC3: u8 = 1;
        pub const SLTI_FUNC3: u8 = 2;
        pub const SLTIU_FUNC3: u8 = 3;
        pub const XORI_FUNC3: u8 = 4;
        pub const SRLI_SRAI_FUNC3: u8 = 5;
        pub const ORI_FUNC3: u8 = 6;
        pub const ANDI_FUNC3: u8 = 7;
    });
    instruction!(OpAmo, 30, Size::Word, TypeR, {
        pub const ADD_FUNCT10: u16 = 0;
        pub const SUB_FUNCT10: u16 = 1;
        pub const SLL_FUNCT10: u16 = 2;
        pub const SLT_FUNCT10: u16 = 3;
        pub const SLTU_FUNCT10: u16 = 4;
        pub const XOR_FUNCT10: u16 = 5;
        pub const SRL_FUNCT10: u16 = 6;
        pub const SRA_FUNCT10: u16 = 7;
        pub const OR_FUNCT10: u16 = 8;
        pub const AND_FUNCT10: u16 = 9;
        pub const MUL_FUNCT10: u16 = 10;
        pub const MULH_FUNCT10: u16 = 11;
        pub const MULHSU_FUNCT10: u16 = 12;
        pub const MULHU_FUNCT10: u16 = 13;
        pub const DIV_FUNCT10: u16 = 14;
        pub const DIVU_FUNCT10: u16 = 15;
        pub const REM_FUNCT10: u16 = 16;
        pub const REMU_FUNCT10: u16 = 17;
        pub const LR_FUNCT10: u16 = 18;
        pub const SC_FUNCT10: u16 = 19;
        pub const AMOSWAP_FUNCT10: u16 = 20;
        pub const AMOADD_FUNCT10: u16 = 21;
        pub const AMOXOR_FUNCT10: u16 = 22;
        pub const AMOAND_FUNCT10: u16 = 23;
        pub const AMOOR_FUNCT10: u16 = 24;
        pub const AMOMIN_FUNCT10: u16 = 25;
        pub const AMOMAX_FUNCT10: u16 = 26;
        pub const AMOMINU_FUNCT10: u16 = 27;
        pub const AMOMAXU_FUNCT10: u16 = 28;
    });
    instruction!(SystemMiscMem, 31, Size::Word, TypeI, {
        pub const ECALL_IMM: i32 = 0;
        pub const EBREAK_IMM: i32 = 1;
        pub const FENCEI_IMM: i32 = 2;
        pub const WFI_IMM: i32 = 3;
        pub const MRET_IMM: i32 = 4;
        pub const EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3: u8 = 0;
        pub const CSRRW_FUNCT3: u8 = 1;
        pub const CSRRS_FUNCT3: u8 = 2;
        pub const CSRRC_FUNCT3: u8 = 3;
        pub const CSRRWI_FUNCT3: u8 = 4;
        pub const CSRRSI_FUNCT3: u8 = 5;
        pub const CSRRCI_FUNCT3: u8 = 6;
    });
}

/// RISC-V Instruction
#[cfg(feature = "transpiler")]
pub mod riscv {
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
