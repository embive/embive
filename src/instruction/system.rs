use crate::engine::Engine;
use crate::error::Error;
use crate::instruction::format::TypeI;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::CSOperation;

use super::INSTRUCTION_SIZE;

const ECALL_IMM: i32 = 0x0000;
const EBREAK_IMM: i32 = 0x0001;

const EBREAK_ECALL_FUNCT3: u8 = 0b000;
const CSRRW_FUNCT3: u8 = 0b001;
const CSRRS_FUNCT3: u8 = 0b010;
const CSRRC_FUNCT3: u8 = 0b011;
const CSRRWI_FUNCT3: u8 = 0b101;
const CSRRSI_FUNCT3: u8 = 0b110;
const CSRRCI_FUNCT3: u8 = 0b111;

/// System OpCode
/// Format: I-Type.
/// Action: Halt
pub struct System {}

impl<M: Memory> Instruction<M> for System {
    #[inline(always)]
    fn decode_execute(data: u32, engine: &mut Engine<'_, M>) -> Result<bool, Error> {
        let inst = TypeI::from(data);

        let ret = if inst.funct3 == EBREAK_ECALL_FUNCT3 {
            match inst.imm {
                ECALL_IMM => engine.syscall().map(|_| true), // Execute the syscall function (ecall)
                EBREAK_IMM => Ok(false),                     // Halt the execution (ebreak)
                _ => return Err(Error::InvalidInstruction),
            }
        } else {
            let op = match inst.funct3 {
                CSRRW_FUNCT3 => Some(CSOperation::Write(
                    engine.registers.cpu.get(inst.rs1)? as u32
                )),
                CSRRS_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Set(engine.registers.cpu.get(inst.rs1)? as u32))
                    } else {
                        None
                    }
                }
                CSRRC_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Clear(
                            engine.registers.cpu.get(inst.rs1)? as u32
                        ))
                    } else {
                        None
                    }
                }
                CSRRWI_FUNCT3 => Some(CSOperation::Write(inst.rs1 as u32)),
                CSRRSI_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Set(inst.rs1 as u32))
                    } else {
                        None
                    }
                }
                CSRRCI_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Clear(inst.rs1 as u32))
                    } else {
                        None
                    }
                }
                _ => return Err(Error::InvalidInstruction),
            };

            let res = engine
                .registers
                .control_status
                .operation(op, (inst.imm & 0b1111_1111_1111) as u16)?;

            if inst.rd != 0 {
                let rd = engine.registers.cpu.get_mut(inst.rd)?;
                *rd = res as i32;
            }

            Ok(true)
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
        assert_eq!(result, Err(Error::NoSyscallFunction));
    }

    #[test]
    fn test_ecall() {
        let syscall_fn: SyscallFn<SliceMemory<'_>> = |nr, args, memory| {
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

    #[test]
    fn test_csrrw() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        *engine.registers.cpu.get_mut(1).unwrap() = 0x1234;
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let csrrw = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x340,
            funct3: CSRRW_FUNCT3,
        };

        let result = System::decode_execute(csrrw.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrs() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1230)), 0x340)
            .unwrap();

        *engine.registers.cpu.get_mut(1).unwrap() = 0x1234;
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let csrrs = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x340,
            funct3: CSRRS_FUNCT3,
        };

        let result = System::decode_execute(csrrs.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0x1230);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrc() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x340)
            .unwrap();

        *engine.registers.cpu.get_mut(1).unwrap() = 0x1230;
        *engine.registers.cpu.get_mut(2).unwrap() = 0x1230;

        let csrrc = TypeI {
            rd: 1,
            rs1: 2,
            imm: 0x340,
            funct3: CSRRC_FUNCT3,
        };

        let result = System::decode_execute(csrrc.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0x1234);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x4
        );
    }

    #[test]
    fn test_csrrwi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        *engine.registers.cpu.get_mut(1).unwrap() = 0x1234;

        let csrrwi = TypeI {
            rd: 1,
            rs1: 1,
            imm: 0x340,
            funct3: CSRRWI_FUNCT3,
        };

        let result = System::decode_execute(csrrwi.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x1
        );
    }

    #[test]
    fn test_csrrsi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1230)), 0x340)
            .unwrap();

        *engine.registers.cpu.get_mut(1).unwrap() = 0x1234;

        let csrrsi = TypeI {
            rd: 1,
            rs1: 4,
            imm: 0x340,
            funct3: CSRRSI_FUNCT3,
        };

        let result = System::decode_execute(csrrsi.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0x1230);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrci() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x340)
            .unwrap();

        *engine.registers.cpu.get_mut(1).unwrap() = 0x1230;

        let csrrci = TypeI {
            rd: 1,
            rs1: 4,
            imm: 0x340,
            funct3: CSRRCI_FUNCT3,
        };

        let result = System::decode_execute(csrrci.into(), &mut engine);
        assert_eq!(result, Ok(true));
        assert_eq!(engine.registers.cpu.get(1).unwrap(), 0x1234);
        assert_eq!(
            engine
                .registers
                .control_status
                .operation(None, 0x340)
                .unwrap(),
            0x1230
        );
    }
}
