//! RISC-V Instruction Formats
//! Source: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf>

/// R-Type Instruction Format
#[doc = include_str!("../../assets/formats/r-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeR {
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub funct10: u16,
}

impl From<u32> for TypeR {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeR {
            rd: ((inst >> 7) & 0b1_1111) as usize,
            rs1: ((inst >> 15) & 0b1_1111) as usize,
            rs2: ((inst >> 20) & 0b1_1111) as usize,
            funct10: (((inst >> 22) & (0b111_1111 << 3)) | ((inst >> 12) & 0b111)) as u16,
        }
    }
}

impl From<TypeR> for u32 {
    fn from(ty: TypeR) -> u32 {
        ((ty.rd as u32) << 7)
            | (((ty.funct10 as u32) & 0b111) << 12)
            | (((ty.funct10 as u32) & (0b111_1111 << 3)) << 22)
            | ((ty.rs1 as u32) << 15)
            | ((ty.rs2 as u32) << 20)
    }
}

/// I-Type Instruction Format
#[doc = include_str!("../../assets/formats/i-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeI {
    pub rd: usize,
    pub rs1: usize,
    pub imm: i32,
    pub funct3: u8,
}

impl From<u32> for TypeI {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeI {
            rd: ((inst >> 7) & 0b1_1111) as usize,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as usize,
            imm: ((inst & (0b1111_1111_1111 << 20)) as i32 >> 20),
        }
    }
}

impl From<TypeI> for u32 {
    fn from(ty: TypeI) -> u32 {
        ((ty.rd as u32) << 7)
            | ((ty.funct3 as u32) << 12)
            | ((ty.rs1 as u32) << 15)
            | ((ty.imm as u32 & 0b1111_1111_1111) << 20)
    }
}

/// S-Type Instruction Format
#[doc = include_str!("../../assets/formats/s-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeS {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: i32,
    pub funct3: u8,
}

impl From<u32> for TypeS {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeS {
            imm: ((inst & (0b111_1111 << 25)) | ((inst & (0b1_1111 << 7)) << 13)) as i32 >> 20,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as usize,
            rs2: ((inst >> 20) & 0b1_1111) as usize,
        }
    }
}

impl From<TypeS> for u32 {
    fn from(ty: TypeS) -> u32 {
        ((ty.imm as u32 & 0b1_1111) << 7)
            | (((ty.imm as u32) & (0b111_1111 << 5)) << 20)
            | ((ty.funct3 as u32) << 12)
            | ((ty.rs1 as u32) << 15)
            | ((ty.rs2 as u32) << 20)
    }
}

/// B-Type Instruction Format
#[doc = include_str!("../../assets/formats/b-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeB {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: i32,
    pub funct3: u8,
}

impl From<u32> for TypeB {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeB {
            imm: ((inst & (0b1 << 31))
                | ((inst & (0b1 << 7)) << 23)
                | ((inst & (0b11_1111 << 25)) >> 1)
                | ((inst & (0b1111 << 8)) << 12)) as i32
                >> 19,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as usize,
            rs2: ((inst >> 20) & 0b1_1111) as usize,
        }
    }
}

impl From<TypeB> for u32 {
    fn from(ty: TypeB) -> u32 {
        ((ty.imm as u32 & 0b1_1110) << 7)
            | (((ty.imm as u32) & (0b1 << 11)) >> 4)
            | (((ty.imm as u32) & (0b11_1111 << 5)) << 20)
            | (((ty.imm as u32) & (0b1 << 12)) << 19)
            | ((ty.funct3 as u32) << 12)
            | ((ty.rs1 as u32) << 15)
            | ((ty.rs2 as u32) << 20)
    }
}

/// U-Type Instruction Format
#[doc = include_str!("../../assets/formats/u-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeU {
    pub imm: i32,
    pub rd: usize,
}

impl From<u32> for TypeU {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeU {
            rd: ((inst >> 7) & 0b11111) as usize,
            imm: (inst & (0b1111_1111_1111_1111_1111 << 12)) as i32,
        }
    }
}

impl From<TypeU> for u32 {
    fn from(ty: TypeU) -> u32 {
        ((ty.rd as u32) << 7) | (ty.imm as u32 & (0b1111_1111_1111_1111_1111 << 12))
    }
}

/// J-Type Instruction Format
#[doc = include_str!("../../assets/formats/j-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeJ {
    pub imm: i32,
    pub rd: usize,
}

impl From<u32> for TypeJ {
    #[inline(always)]
    fn from(inst: u32) -> Self {
        TypeJ {
            rd: ((inst >> 7) & 0b1_1111) as usize,
            imm: ((inst & (0b1 << 31))
                | ((inst & (0b1111_1111 << 12)) << 11)
                | ((inst & (0b1 << 20)) << 2)
                | ((inst & (0b11_1111_1111 << 21)) >> 9)) as i32
                >> 11,
        }
    }
}

impl From<TypeJ> for u32 {
    fn from(ty: TypeJ) -> u32 {
        ((ty.rd as u32) << 7)
            | ((ty.imm as u32) & (0b1111_1111 << 12))
            | (((ty.imm as u32) & (0b1 << 11)) << 9)
            | (((ty.imm as u32) & (0b11_1111_1111 << 1)) << 20)
            | (((ty.imm as u32) & (0b1 << 20)) << 11)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_from_into<T>(inst: T)
    where
        T: Into<u32> + From<u32> + PartialEq + std::fmt::Debug + Copy,
    {
        let into: u32 = inst.into();
        let from: T = T::from(into);
        assert_eq!(inst, from);
    }

    #[test]
    fn test_type_r() {
        let inst = u32::from_le(0b01000000001100100101000010110011); // sra x1, x4, x3
        let parsed = TypeR::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.rd, 1);
        assert_eq!(parsed.rs1, 4);
        assert_eq!(parsed.rs2, 3);
        assert_eq!(parsed.funct10, (32 << 3) | 5);
    }

    #[test]
    fn test_type_i_negative() {
        let inst = u32::from_le(0b11000001100000010000000110010011); // addi x3, x2, -1000
        let parsed = TypeI::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.rd, 3);
        assert_eq!(parsed.funct3, 0);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.imm as i32, -1000);
    }

    #[test]
    fn test_type_i_positive() {
        let inst = u32::from_le(0b01111111101000000100000010010011); // xori x1, x0, 2042
        let parsed = TypeI::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.rd, 1);
        assert_eq!(parsed.funct3, 4);
        assert_eq!(parsed.rs1, 0);
        assert_eq!(parsed.imm as i32, 2042);
    }

    #[test]
    fn test_type_s_negative() {
        let inst = u32::from_le(0b11100000000100010001101100100011); // sh x1, -490(x2)
        let parsed = TypeS::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, -490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_s_positive() {
        let inst = u32::from_le(0b00011110000100010001010100100011); // sh x1, 490(x2)
        let parsed = TypeS::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, 490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_b_negative() {
        let inst = u32::from_le(0b10101100100000101001010011100011); // bne x5, x8, -1336
        let parsed = TypeB::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, -1336);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_b_positive() {
        let inst = u32::from_le(0b00101100100000101001010001100011); // bne x5, x8, 712
        let parsed = TypeB::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, 712);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_u_negative() {
        let inst = u32::from_le(0b11110000001000001111000110110111); // lui x3, -65009
        let parsed = TypeU::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, -65009 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_u_positive() {
        let inst = u32::from_le(0b00010000001000001111000110110111); // lui x3, 66063
        let parsed = TypeU::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, 66063 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_negative() {
        let inst = u32::from_le(0b10101100001100011011000111101111); // jal x3, -935230
        let parsed = TypeJ::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, -935230);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_positive() {
        let inst = u32::from_le(0b01011100001100011011000111101111); // jal x3, 114114
        let parsed = TypeJ::from(inst);
        test_from_into(parsed);

        assert_eq!(parsed.imm, 114114);
        assert_eq!(parsed.rd, 3);
    }
}
