//! RISC-V Instruction Formats
//! Source: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf>

/// R-Type Instruction Format
#[doc = include_str!("../../assets/formats/r-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeR {
    pub rd: u8,
    pub funct3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub funct7: u8,
}

impl From<u32> for TypeR {
    fn from(inst: u32) -> Self {
        TypeR {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
            funct7: ((inst >> 25) & 0b111_1111) as u8,
        }
    }
}

/// I-Type Instruction Format
#[doc = include_str!("../../assets/formats/i-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeI {
    pub rd: u8,
    pub funct3: u8,
    pub rs1: u8,
    pub imm: i16,
}

impl From<u32> for TypeI {
    fn from(inst: u32) -> Self {
        TypeI {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            imm: ((inst & (0b1111_1111_1111 << 20)) as i32 >> 20) as i16,
        }
    }
}

/// S-Type Instruction Format
#[doc = include_str!("../../assets/formats/s-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeS {
    pub imm: i16,
    pub funct3: u8,
    pub rs1: u8,
    pub rs2: u8,
}

impl From<u32> for TypeS {
    fn from(inst: u32) -> Self {
        TypeS {
            imm: (((inst & (0b111_1111 << 25)) | ((inst & (0b1_1111 << 7)) << 13)) as i32 >> 20)
                as i16,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
        }
    }
}

/// B-Type Instruction Format
#[doc = include_str!("../../assets/formats/b-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeB {
    pub imm: i16,
    pub funct3: u8,
    pub rs1: u8,
    pub rs2: u8,
}

impl From<u32> for TypeB {
    fn from(inst: u32) -> Self {
        TypeB {
            imm: (((inst & (0b1 << 31))
                | ((inst & (0b1 << 7)) << 23)
                | ((inst & (0b11_1111 << 25)) >> 1)
                | ((inst & (0b1111 << 8)) << 12)) as i32
                >> 19) as i16,
            funct3: ((inst >> 12) & 0b111) as u8,
            rs1: ((inst >> 15) & 0b1_1111) as u8,
            rs2: ((inst >> 20) & 0b1_1111) as u8,
        }
    }
}

/// U-Type Instruction Format
#[doc = include_str!("../../assets/formats/u-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeU {
    pub rd: u8,
    pub imm: i32,
}

impl From<u32> for TypeU {
    fn from(inst: u32) -> Self {
        TypeU {
            rd: ((inst >> 7) & 0b11111) as u8,
            imm: (inst & (0b1111_1111_1111_1111_1111 << 12)) as i32,
        }
    }
}

/// J-Type Instruction Format
#[doc = include_str!("../../assets/formats/j-type.svg")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TypeJ {
    pub rd: u8,
    pub imm: i32,
}

impl From<u32> for TypeJ {
    fn from(inst: u32) -> Self {
        TypeJ {
            rd: ((inst >> 7) & 0b1_1111) as u8,
            imm: ((inst & (0b1 << 31))
                | ((inst & (0b1111_1111 << 12)) << 11)
                | ((inst & (0b1 << 20)) << 2)
                | ((inst & (0b11_1111_1111 << 21)) >> 9)) as i32
                >> 11,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_r() {
        let inst = u32::from_le(0b01000000001100100101000010110011); // sra x1, x4, x3
        let parsed = TypeR::from(inst);

        assert_eq!(parsed.rd, 1);
        assert_eq!(parsed.funct3, 5);
        assert_eq!(parsed.rs1, 4);
        assert_eq!(parsed.rs2, 3);
        assert_eq!(parsed.funct7, 32);
    }

    #[test]
    fn test_type_i_negative() {
        let inst = u32::from_le(0b11000001100000010000000110010011); // addi x3, x2, -1000
        let parsed = TypeI::from(inst);

        assert_eq!(parsed.rd, 3);
        assert_eq!(parsed.funct3, 0);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.imm as i32, -1000);
    }

    #[test]
    fn test_type_i_positive() {
        let inst = u32::from_le(0b01111111101000000100000010010011); // xori x1, x0, 2042
        let parsed = TypeI::from(inst);

        assert_eq!(parsed.rd, 1);
        assert_eq!(parsed.funct3, 4);
        assert_eq!(parsed.rs1, 0);
        assert_eq!(parsed.imm as i32, 2042);
    }

    #[test]
    fn test_type_s_negative() {
        let inst = u32::from_le(0b11100000000100010001101100100011); // sh x1, -490(x2)
        let parsed = TypeS::from(inst);

        assert_eq!(parsed.imm, -490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_s_positive() {
        let inst = u32::from_le(0b00011110000100010001010100100011); // sh x1, 490(x2)
        let parsed = TypeS::from(inst);

        assert_eq!(parsed.imm, 490);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 2);
        assert_eq!(parsed.rs2, 1);
    }

    #[test]
    fn test_type_b_negative() {
        let inst = u32::from_le(0b10101100100000101001010011100011); // bne x5, x8, -1336
        let parsed = TypeB::from(inst);

        assert_eq!(parsed.imm, -1336);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_b_positive() {
        let inst = u32::from_le(0b00101100100000101001010001100011); // bne x5, x8, 712
        let parsed = TypeB::from(inst);

        assert_eq!(parsed.imm, 712);
        assert_eq!(parsed.funct3, 1);
        assert_eq!(parsed.rs1, 5);
        assert_eq!(parsed.rs2, 8);
    }

    #[test]
    fn test_type_u_negative() {
        let inst = u32::from_le(0b11110000001000001111000110110111); // lui x3, -65009
        let parsed = TypeU::from(inst);

        assert_eq!(parsed.imm, -65009 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_u_positive() {
        let inst = u32::from_le(0b00010000001000001111000110110111); // lui x3, 66063
        let parsed = TypeU::from(inst);

        assert_eq!(parsed.imm, 66063 << 12);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_negative() {
        let inst = u32::from_le(0b10101100001100011011000111101111); // jal x3, -935230
        let parsed = TypeJ::from(inst);

        assert_eq!(parsed.imm, -935230);
        assert_eq!(parsed.rd, 3);
    }

    #[test]
    fn test_type_j_positive() {
        let inst = u32::from_le(0b01011100001100011011000111101111); // jal x3, 114114
        let parsed = TypeJ::from(inst);

        assert_eq!(parsed.imm, 114114);
        assert_eq!(parsed.rd, 3);
    }
}
