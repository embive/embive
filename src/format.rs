//! RISC-V & Embive Instruction Formats
use core::fmt;

pub(crate) const COMPRESSED_REGISTER_OFFSET: u8 = 8;

/// Instruction Format Trait
/// Each format can be expressed as a raw RISC-V bytecode, a raw Embive bytecode, and a struct.
#[allow(dead_code)]
pub trait Format: fmt::Debug + PartialEq + Copy + Clone {
    /// Decode the instruction from a raw RISC-V bytecode
    fn from_riscv(inst: u32) -> Self;
    /// Decode the instruction from a raw Embive bytecode
    fn from_embive(inst: u32) -> Self;
    /// Encode the instruction into a raw Embive bytecode
    fn to_embive(self) -> u32;
}

/// R-Type Instruction Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeR {
    /// Destination Register
    pub rd: u8,
    /// Source Register 1
    pub rs1: u8,
    /// Source Register 2
    pub rs2: u8,
    /// Function Type
    pub funct10: u16,
}

impl Format for TypeR {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeR {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
            funct10: (((inst >> 22) & (0b111_1111 << 3)) | ((inst >> 12) & 0b111)) as u16,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeR {
            rd: ((inst >> 17) & 0b1_1111) as u8,
            rs1: ((inst >> 22) & 0b1_1111) as u8,
            rs2: ((inst >> 27) & 0b1_1111) as u8,
            funct10: ((inst >> 7) & 0b11_1111_1111) as u16,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd as u32) << 17)
            | ((self.rs1 as u32) << 22)
            | ((self.rs2 as u32) << 27)
            | ((self.funct10 as u32) << 7)
    }
}

/// I-Type Instruction Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeI {
    /// Destination Register / Source Register 2
    pub rd_rs2: u8,
    /// Source Register 1
    pub rs1: u8,
    /// Immediate Value
    pub imm: i32,
    /// Function Type
    pub funct3: u8,
}

impl Format for TypeI {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeI {
            rd_rs2: ((inst >> 7) & 0b1_1111) as u8,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            imm: ((inst & (0b1111_1111_1111 << 20)) as i32 >> 20),
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeI {
            rd_rs2: ((inst >> 10) & 0b1_1111) as u8,
            funct3: ((inst >> 7) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            imm: ((inst & (0b1111_1111_1111 << 20)) as i32 >> 20),
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs2 as u32) << 10)
            | ((self.funct3 as u32) << 7)
            | ((self.rs1 as u32) << 15)
            | ((self.imm as u32 & 0b1111_1111_1111) << 20)
    }
}

/// S-Type Instruction Format (RISC-V only, this is converted to TypeI)
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeS {
    /// Source Register 1
    pub rs1: u8,
    /// Source Register 2
    pub rs2: u8,
    /// Immediate Value
    pub imm: i32,
    /// Function Type
    pub funct3: u8,
}

impl Format for TypeS {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeS {
            imm: ((inst & (0b111_1111 << 25)) | ((inst & (0b1_1111 << 7)) << 13)) as i32 >> 20,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeS {
            imm: (inst & (0b1111_1111_1111 << 20)) as i32 >> 20,
            funct3: ((inst >> 7) & 0b111) as u8,
            rs2: ((inst >> 10) & 0b1_1111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.imm as u32) << 20)
            | ((self.funct3 as u32) << 7)
            | ((self.rs2 as u32) << 10)
            | ((self.rs1 as u32) << 15)
    }
}

/// B-Type Instruction Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeB {
    /// Source Register 1
    pub rs1: u8,
    /// Source Register 2
    pub rs2: u8,
    /// Immediate Value
    pub imm: i32,
    /// Function Type
    pub funct3: u8,
}

impl Format for TypeB {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeB {
            imm: ((inst & (0b1 << 31))
                | ((inst & (0b1 << 7)) << 23)
                | ((inst & (0b11_1111 << 25)) >> 1)
                | ((inst & (0b1111 << 8)) << 12)) as i32
                >> 19,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeB {
            imm: (inst & (0b1111_1111_1111 << 20)) as i32 >> 19,
            funct3: ((inst >> 7) & 0b111) as u8,
            rs1: ((inst >> 10) & 0b1_1111) as u8,
            rs2: ((inst >> 15) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.imm as u32) << 19)
            | ((self.funct3 as u32) << 7)
            | ((self.rs1 as u32) << 10)
            | ((self.rs2 as u32) << 15)
    }
}

/// U-Type Instruction Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeU {
    /// Destination Register
    pub rd: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeU {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeU {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            imm: (inst & (0b1111_1111_1111_1111_1111 << 12)) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        Self::from_riscv(inst)
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd as u32) << 7) | (self.imm as u32 & (0b1111_1111_1111_1111_1111 << 12))
    }
}

/// J-Type Instruction Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeJ {
    /// Destination Register
    pub rd: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeJ {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeJ {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            imm: ((inst & (0b1 << 31))
                | ((inst & (0b1111_1111 << 12)) << 11)
                | ((inst & (0b1 << 20)) << 2)
                | ((inst & (0b11_1111_1111 << 21)) >> 9)) as i32
                >> 11,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeJ {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            imm: (inst & (0b1111_1111_1111_1111_1111 << 12)) as i32 >> 11,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd as u32) << 7) | ((self.imm as u32) << 11)
    }
}

/// CIW Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCIW {
    /// Destination Register
    pub rd: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCIW {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCIW {
            rd: (((inst >> 2) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: (((inst & (0b1 << 6)) >> 4)
                | ((inst & (0b1 << 5)) >> 2)
                | ((inst & (0b11 << 11)) >> 7)
                | ((inst & (0b1111 << 7)) >> 1)) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCIW {
            rd: (((inst >> 5) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: ((inst & (0b1111_1111 << 8)) >> 6) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd.wrapping_sub(COMPRESSED_REGISTER_OFFSET) as u32) << 5)
            | (((self.imm as u32) << 6) & (0b1111_1111 << 8))
    }
}

/// CL Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCL {
    /// Destination Register / Source Register 2
    pub rd_rs2: u8,
    /// Source Register 1
    pub rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCL {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCL {
            rd_rs2: (((inst >> 2) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: (((inst & (0b1 << 5)) << 1)
                | ((inst & (0b111 << 10)) >> 7)
                | ((inst & (0b1 << 6)) >> 4)) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCL {
            rd_rs2: (((inst >> 5) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            rs1: (((inst >> 8) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: ((inst & (0b1_1111 << 11)) >> 9) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs2.wrapping_sub(COMPRESSED_REGISTER_OFFSET) as u32) << 5)
            | ((self.rs1.wrapping_sub(COMPRESSED_REGISTER_OFFSET) as u32) << 8)
            | ((self.imm as u32) << 9)
    }
}

/// CI Format 1 (IMM[5:0])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCI1 {
    /// Destination Register / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCI1 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCI1 {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            imm: ((((inst & (0b1_1111 << 2)) | ((inst & (0b1 << 12)) >> 5)) as i8) >> 2) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCI1 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: (((inst & (0b11_1111 << 10)) as i16) >> 10) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | (((self.imm as u32) << 10) & (0b11_1111 << 10))
    }
}

/// CI Format 2 (IMM[9:4])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCI2 {
    /// Destination Register / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCI2 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCI2 {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            imm: (((((inst & (0b1 << 12)) >> 5)
                | ((inst & (0b11 << 3)) << 2)
                | ((inst & (0b1 << 5)) >> 1)
                | ((inst & (0b1 << 2)) << 1)
                | ((inst & (0b1 << 6)) >> 4)) as i8) as i32)
                << 2,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCI2 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: (((inst & (0b11_1111 << 10)) as i16) >> 6) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | (((self.imm as u32) << 6) & (0b11_1111 << 10))
    }
}

/// CI Format 3 (IMM[17:12])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCI3 {
    /// Destination Register / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCI3 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCI3 {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            imm: ((((inst & (0b1_1111 << 2)) | ((inst & (0b1 << 12)) >> 5)) as i8) as i32) << 10,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCI3 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: (((inst & (0b11_1111 << 10)) as i16) as i32) << 2,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | (((self.imm as u32) >> 2) & (0b11_1111 << 10))
    }
}

/// CI Format 4 (UIMM[5:0])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCI4 {
    /// Destination Register / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCI4 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCI4 {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            imm: (((inst & (0b1_1111 << 2)) | ((inst & (0b1 << 12)) >> 5)) >> 2) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCI4 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: ((inst & (0b11_1111 << 10)) >> 10) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.imm as u32) << 10)
    }
}

/// CI Format 5 (UIMM[7:2])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCI5 {
    /// Destination Register / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCI5 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCI5 {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            imm: ((((inst & (0b11 << 2)) << 11)
                | (inst & (0b1 << 12))
                | ((inst & (0b111 << 4)) << 5)) as i32)
                >> 7,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCI5 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: ((inst & (0b11_1111 << 10)) >> 8) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.imm as u32) << 8)
    }
}

/// CB Format 1 (UIMM[5:0])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCB1 {
    /// Destination / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCB1 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCB1 {
            rd_rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: (((inst & (0b1_1111 << 2)) | ((inst & (0b1 << 12)) >> 5)) >> 2) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCB1 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: ((inst & (0b11_1111 << 10)) >> 10) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.imm as u32) << 10)
    }
}

/// CB Format 2 (IMM[5:0])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCB2 {
    /// Destination / Source Register 1
    pub rd_rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCB2 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCB2 {
            rd_rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: ((((inst & (0b1_1111 << 2)) | ((inst & (0b1 << 12)) >> 5)) as i8) >> 2) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCB2 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            imm: (((inst & (0b11_1111 << 10)) as i16) >> 10) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | (((self.imm as u32) << 10) & (0b11_1111 << 10))
    }
}

/// CB Format 3 (rs2)
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCB3 {
    /// Destination / Source Register 1
    pub rd_rs1: u8,
    /// Source Register 2
    pub rs2: u8,
}

impl Format for TypeCB3 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCB3 {
            rd_rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            rs2: (((inst >> 2) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCB3 {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            rs2: ((inst >> 10) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.rs2 as u32) << 10)
    }
}

/// CB Format 4 (IMM[8:1])
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCB4 {
    /// Source Register 1
    pub rs1: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCB4 {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCB4 {
            rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: (((((inst & (0b1 << 12)) >> 5)
                | (inst & (0b11 << 5))
                | ((inst & (0b1 << 2)) << 2)
                | ((inst & (0b11 << 10)) >> 8)
                | ((inst & (0b11 << 3)) >> 3)) as i8) as i32)
                << 1,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCB4 {
            rs1: (((inst >> 5) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            imm: (((inst & (0b1111_1111 << 8)) as i16) >> 7) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rs1.wrapping_sub(COMPRESSED_REGISTER_OFFSET) as u32) << 5)
            | (((self.imm as u32) << 7) & (0b1111_1111 << 8))
    }
}

/// CR Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCR {
    /// Destination / Source Register 1
    pub rd_rs1: u8,
    /// Source Register 2
    pub rs2: u8,
}

impl Format for TypeCR {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCR {
            rd_rs1: ((inst >> 7) & 0b1_1111) as u8,
            rs2: ((inst >> 2) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCR {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            rs2: ((inst >> 10) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.rs2 as u32) << 10)
    }
}

/// CS Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCS {
    /// Destination / Source Register 1
    pub rd_rs1: u8,
    /// Source Register 2
    pub rs2: u8,
}

impl Format for TypeCS {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCS {
            rd_rs1: (((inst >> 7) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
            rs2: (((inst >> 2) & 0b111) as u8).wrapping_add(COMPRESSED_REGISTER_OFFSET),
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCS {
            rd_rs1: ((inst >> 5) & 0b1_1111) as u8,
            rs2: ((inst >> 10) & 0b1_1111) as u8,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rd_rs1 as u32) << 5) | ((self.rs2 as u32) << 10)
    }
}

/// CSS Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCSS {
    /// Source Register 2
    pub rs2: u8,
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCSS {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCSS {
            rs2: ((inst >> 2) & 0b1_1111) as u8,
            imm: (((inst & (0b11 << 7)) | ((inst & (0b1111 << 9)) >> 6)) >> 1) as i32,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCSS {
            rs2: ((inst >> 5) & 0b1_1111) as u8,
            imm: ((inst & (0b11_1111 << 10)) >> 8) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.rs2 as u32) << 5) | ((self.imm as u32) << 8)
    }
}

/// CJ Format
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeCJ {
    /// Immediate Value
    pub imm: i32,
}

impl Format for TypeCJ {
    #[inline(always)]
    fn from_riscv(inst: u32) -> Self {
        TypeCJ {
            imm: (((((inst & (0b1 << 12)) << 3)
                | ((inst & (0b1 << 8)) << 6)
                | ((inst & (0b11 << 9)) << 3)
                | ((inst & (0b1 << 6)) << 5)
                | ((inst & (0b1 << 7)) << 3)
                | ((inst & (0b1 << 2)) << 7)
                | ((inst & (0b1 << 11)) >> 3)
                | ((inst & (0b111 << 3)) << 2)) as i16) as i32)
                >> 4,
        }
    }

    #[inline(always)]
    fn from_embive(inst: u32) -> Self {
        TypeCJ {
            imm: (((inst & (0b111_1111_1111 << 5)) as i16) >> 4) as i32,
        }
    }

    #[inline(always)]
    fn to_embive(self) -> u32 {
        ((self.imm as u32) << 4) & (0b111_1111_1111 << 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_from_to<T>(inst: T)
    where
        T: Format + PartialEq + std::fmt::Debug + Copy,
    {
        let into: u32 = inst.to_embive();
        let from: T = T::from_embive(into);
        assert_eq!(inst, from);
    }

    #[test]
    fn test_type_r() {
        let inst = 0b01000000001100100101000010110011; // sra x1, x4, x3
        let parsed = TypeR::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd, 1);
        assert_eq!(parsed.rs1, 4);
        assert_eq!(parsed.rs2, 3);
        assert_eq!(parsed.funct10, (32 << 3) | 5);
    }

    #[test]
    fn test_type_i_negative() {
        let inst = 0b11000001100000010000000110010011; // addi x3, x2, -1000
        let parsed = TypeI::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs2, 3);
        assert_eq!(parsed.funct3, 0);
        assert_eq!(parsed.rs1, 2);
        assert_eq!({ parsed.imm }, -1000);
    }

    #[test]
    fn test_type_i_positive() {
        let inst = 0b01111111101000000100000010010011; // xori x1, x0, 2042
        let parsed = TypeI::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs2, 1);
        assert_eq!(parsed.funct3, 4);
        assert_eq!(parsed.rs1, 0);
        assert_eq!({ parsed.imm }, 2042);
    }

    #[test]
    fn test_type_s_negative() {
        let inst = 0b11100000000100010001101100100011; // sh x1, -490(x2)
        let parsed = TypeS::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, -490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_s_positive() {
        let inst = 0b00011110000100010001010100100011; // sh x1, 490(x2)
        let parsed = TypeS::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, 490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_b_negative() {
        let inst = 0b10101100100000101001010011100011; // bne x5, x8, -1336
        let parsed = TypeB::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, -1336);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_b_positive() {
        let inst = 0b00101100100000101001010001100011; // bne x5, x8, 712
        let parsed = TypeB::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, 712);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_u_negative() {
        let inst = 0b11110000001000001111000110110111; // lui x3, -65009
        let parsed = TypeU::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, -65009 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_u_positive() {
        let inst = 0b00010000001000001111000110110111; // lui x3, 66063
        let parsed = TypeU::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, 66063 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_negative() {
        let inst = 0b10101100001100011011000111101111; // jal x3, -935230
        let parsed = TypeJ::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, -935230);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_positive() {
        let inst = 0b01011100001100011011000111101111; // jal x3, 114114
        let parsed = TypeJ::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, 114114);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_ciw() {
        let inst = 0b0001011011001000; // c.addi4spn x10, 868
        let parsed = TypeCIW::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd, 10);
        assert_eq!(parsed.imm, 868);
    }

    #[test]
    fn test_type_cl() {
        let inst = 0b0101100011010100; // c.lw x13, 52(x9)
        let parsed = TypeCL::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs2, 13);
        assert_eq!(parsed.rs1, 9);
        assert_eq!(parsed.imm, 52);
    }

    #[test]
    fn test_type_ci1() {
        let inst = 0b0100010101010101; // c.li x10, 21
        let parsed = TypeCI1::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 21);
    }

    #[test]
    fn test_type_ci1_negative() {
        let inst = 0b0101010100101101; // c.li x10, -21
        let parsed = TypeCI1::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, -21);
    }

    #[test]
    fn test_type_ci2() {
        let inst = 0b0110000101011001; // c.addi16sp 400
        let parsed = TypeCI2::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 2);
        assert_eq!(parsed.imm, 400);
    }

    #[test]
    fn test_type_ci2_negative() {
        let inst = 0b0111000100100101; // c.addi16sp -416
        let parsed = TypeCI2::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 2);
        assert_eq!(parsed.imm, -416);
    }

    #[test]
    fn test_type_ci3() {
        let inst = 0b0110010101010101; // c.lui x10, 21
        let parsed = TypeCI3::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 21 << 12);
    }

    #[test]
    fn test_type_ci3_negative() {
        let inst = 0b0111010100101101; // c.lui x10, -21
        let parsed = TypeCI3::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, -21 << 12);
    }

    #[test]
    fn test_type_ci4() {
        let inst = 0b0001010100100110; // c.slli x10, 41
        let parsed = TypeCI4::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 41);
    }

    #[test]
    fn test_type_ci5() {
        let inst = 0b0101010101010110; // c.lwsp x10, 116
        let parsed = TypeCI5::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 116);
    }

    #[test]
    fn test_type_cb1() {
        let inst = 0b0001010100100110; // c.slli x10, 41
        let parsed = TypeCB1::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 41);
    }

    #[test]
    fn test_type_cb2() {
        let inst = 0b1000100101101001; // c.andi x10, 26
        let parsed = TypeCB2::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, 26);
    }

    #[test]
    fn test_type_cb2_negative() {
        let inst = 0b1001100100100101; // c.andi x10, -23
        let parsed = TypeCB2::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.imm, -23);
    }

    #[test]
    fn test_type_cb3() {
        let inst = 0b1000110100101101; // c.xor x10, x11
        let parsed = TypeCB3::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.rs2, 11);
    }

    #[test]
    fn test_type_cb4() {
        let inst = 0b1110110101111101; // c.bnez x10, 254
        let parsed = TypeCB4::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rs1, 10);
        assert_eq!(parsed.imm, 254);
    }

    #[test]
    fn test_type_cb4_negative() {
        let inst = 0b1111100100110101; // c.bnez x10, -140
        let parsed = TypeCB4::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rs1, 10);
        assert_eq!(parsed.imm, -140);
    }

    #[test]
    fn test_type_cr() {
        let inst = 0b1000010100101110; // c.mv x10, x11
        let parsed = TypeCR::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rd_rs1, 10);
        assert_eq!(parsed.rs2, 11);
    }

    #[test]
    fn test_type_css() {
        let inst = 0b1101010110101010; // c.swsp x10, 232
        let parsed = TypeCSS::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.rs2, 10);
        assert_eq!(parsed.imm, 232);
    }

    #[test]
    fn test_type_cj() {
        let inst = 0b0011110010101001; // c.jal -1446
        let parsed = TypeCJ::from_riscv(inst);
        test_from_to(parsed);

        assert_eq!(parsed.imm, -1446);
    }
}
