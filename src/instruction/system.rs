use crate::engine::Engine;
use crate::error::EmbiveError;
use crate::instruction::format::TypeI;
use crate::instruction::Instruction;
use crate::memory::Memory;

use super::INSTRUCTION_SIZE;

const ECALL_IMM: i32 = 0x0000;
const EBREAK_IMM: i32 = 0x0001;

const EBREAK_ECALL_FUNCT3: u8 = 0b000;

/// System OpCode
/// Format: I-Type.
/// Action: Halt
pub struct System {}

impl<M: Memory> Instruction<M> for System {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<M>) -> Result<bool, EmbiveError> {
        let inst = TypeI::from(data);

        let ret = match inst.funct3 {
            EBREAK_ECALL_FUNCT3 => {
                match inst.imm {
                    ECALL_IMM => engine.syscall().map(|_| true), // Execute the syscall function (ecall)
                    EBREAK_IMM => Ok(false),                     // Halt the execution (ebreak)
                    _ => Err(EmbiveError::InvalidInstruction),
                }
            }
            _ => Err(EmbiveError::InvalidInstruction),
        };

        // Go to next instruction
        engine.program_counter = engine.program_counter.wrapping_add(INSTRUCTION_SIZE);

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Config, SyscallFn};
    use crate::memory::{Memory, SliceMemory, RAM_OFFSET};
    use crate::registers::CPURegister;

    fn get_ram_addr() -> u32 {
        RAM_OFFSET
    }

    #[test]
    fn test_ebreak() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let misc_mem = TypeI {
            rd: 0,
            rs1: 0,
            imm: 0x1,
            funct3: 0,
        };

        let result = System::decode_execute(misc_mem.into(), &mut engine);
        assert_eq!(result, Ok(false));
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }

    #[test]
    fn test_ecall_error() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        let misc_mem = TypeI {
            rd: 0,
            rs1: 0,
            imm: 0x0,
            funct3: 0,
        };

        let result = System::decode_execute(misc_mem.into(), &mut engine);
        assert_eq!(result, Err(EmbiveError::NoSyscallFunction));
    }

    #[test]
    fn test_ecall() {
        let syscall_fn: SyscallFn<SliceMemory> = |nr, args, memory| {
            assert_eq!(nr, -1);
            for (i, arg) in args.iter().enumerate() {
                assert_eq!(*arg, i as i32);
            }

            memory.store(get_ram_addr(), nr.to_le_bytes()).unwrap();

            Ok(args.iter().sum::<i32>())
        };

        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut engine = Engine::new(
            &mut memory,
            Config {
                syscall_fn: Some(syscall_fn),
                ..Default::default()
            },
        )
        .unwrap();
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A0 as usize)
            .unwrap() = 0;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A1 as usize)
            .unwrap() = 1;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A2 as usize)
            .unwrap() = 2;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A3 as usize)
            .unwrap() = 3;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A4 as usize)
            .unwrap() = 4;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A5 as usize)
            .unwrap() = 5;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A6 as usize)
            .unwrap() = 6;
        *engine
            .registers
            .cpu
            .get_mut(CPURegister::A7 as usize)
            .unwrap() = -1;

        let misc_mem = TypeI {
            rd: 0,
            rs1: 0,
            imm: 0x0,
            funct3: 0,
        };

        let result = System::decode_execute(misc_mem.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(CPURegister::A0 as usize), Ok(0));
        assert_eq!(engine.registers.cpu.get(CPURegister::A1 as usize), Ok(21));
        assert_eq!(
            engine
                .memory
                .load::<4>(get_ram_addr())
                .map(|v| i32::from_le_bytes(v)),
            Ok(-1)
        );
        assert_eq!(engine.program_counter, INSTRUCTION_SIZE);
    }
}
