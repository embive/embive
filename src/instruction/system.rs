use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::{Instruction, Opcode};

use super::INSTRUCTION_SIZE;

const ECALL_IMM: i32 = 0x0000;
const EBREAK_IMM: i32 = 0x0001;

const EBREAK_ECALL_FUNCT3: u8 = 0b000;

/// System OpCode
/// Format: I-Type.
/// Action: Halt
pub struct System {
    ty: TypeI,
}

impl Opcode for System {
    #[inline(always)]
    fn decode(data: u32) -> impl Instruction {
        Self {
            ty: TypeI::from(data),
        }
    }
}

impl Instruction for System {
    #[inline(always)]
    fn execute(&self, engine: &mut Engine) -> Result<bool, EmbiveError> {
        let ret = match self.ty.funct3 {
            EBREAK_ECALL_FUNCT3 => {
                match self.ty.imm {
                    ECALL_IMM => engine.syscall().map(|_| true), // Execute the syscall function (ecall)
                    EBREAK_IMM => Ok(false),                     // Halt the execution (ebreak)
                    _ => Err(EmbiveError::InvalidInstruction),
                }
            }
            _ => Err(EmbiveError::InvalidInstruction),
        };

        // Go to next instruction
        engine.pc = engine.pc.wrapping_add(INSTRUCTION_SIZE);

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::RAM_OFFSET;
    use crate::register::Register;
    use crate::engine::SyscallFn;

    fn get_ram_addr() -> u32 {
        RAM_OFFSET
    }

    #[test]
    fn test_ebreak() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        let misc_mem = System {
            ty: TypeI {
                rd: 0,
                rs1: 0,
                imm: 0x1,
                funct3: 0,
            },
        };

        let result = misc_mem.execute(&mut engine);
        assert_eq!(result, Ok(false));
        assert_eq!(engine.pc, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ecall_error() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        let misc_mem = System {
            ty: TypeI {
                rd: 0,
                rs1: 0,
                imm: 0x0,
                funct3: 0,
            },
        };

        let result = misc_mem.execute(&mut engine);
        assert_eq!(result, Err(EmbiveError::NoSyscallFunction));
    }

    #[test]
    fn test_ecall() {
        let syscall_fn: SyscallFn = |nr, args, memory| {
            assert_eq!(nr, -1);
            for (i, arg) in args.iter().enumerate() {
                assert_eq!(*arg, i as i32);
            }

            memory.store(get_ram_addr(), nr.to_le_bytes()).unwrap();

            (args.iter().sum::<i32>(), 0i32)
        };

        let mut memory = [0; 4];
        let mut engine = Engine::new(&[], &mut memory, Some(syscall_fn)).unwrap();
        *engine.registers.get_mut(Register::A0 as usize).unwrap() = 0;
        *engine.registers.get_mut(Register::A1 as usize).unwrap() = 1;
        *engine.registers.get_mut(Register::A2 as usize).unwrap() = 2;
        *engine.registers.get_mut(Register::A3 as usize).unwrap() = 3;
        *engine.registers.get_mut(Register::A4 as usize).unwrap() = 4;
        *engine.registers.get_mut(Register::A5 as usize).unwrap() = 5;
        *engine.registers.get_mut(Register::A7 as usize).unwrap() = -1;

        let misc_mem = System {
            ty: TypeI {
                rd: 0,
                rs1: 0,
                imm: 0x0,
                funct3: 0,
            },
        };

        let result = misc_mem.execute(&mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.get(Register::A0 as usize), Ok(15));
        assert_eq!(engine.registers.get(Register::A1 as usize), Ok(0));
        assert_eq!(
            engine
                .memory
                .load::<4>(get_ram_addr())
                .map(|v| i32::from_le_bytes(v)),
            Ok(-1)
        );
        assert_eq!(engine.pc, INSTRUCTION_SIZE);
    }
}
