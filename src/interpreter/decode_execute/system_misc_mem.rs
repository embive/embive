use crate::instruction::embive::SystemMiscMem;
use crate::interpreter::{memory::Memory, registers::CSOperation, Error, Interpreter, State};

use super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for SystemMiscMem {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        let ret = if inst.funct3 == Self::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3 {
            match inst.imm {
                Self::ECALL_IMM => Ok(State::Called),  // Syscall (ecall)
                Self::EBREAK_IMM => Ok(State::Halted), // Halt the execution (ebreak)
                Self::FENCEI_IMM => {
                    // Fencing isn't applicable to this implementation.
                    // This is a nop.
                    Ok(State::Running)
                }
                Self::WFI_IMM => Ok(State::Waiting), // Wait for interrupt (wfi)
                Self::MRET_IMM => {
                    // Return from machine-mode trap
                    interpreter.program_counter =
                        interpreter.registers.control_status.trap_return();
                    return Ok(State::Running); // Do not increment the program counter
                }
                _ => return Err(Error::InvalidInstruction(data)),
            }
        } else {
            let op = match inst.funct3 {
                Self::CSRRW_FUNCT3 => Some(CSOperation::Write(
                    interpreter.registers.cpu.get(inst.rs1)? as u32,
                )),
                Self::CSRRS_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Set(
                            interpreter.registers.cpu.get(inst.rs1)? as u32
                        ))
                    } else {
                        None
                    }
                }
                Self::CSRRC_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Clear(
                            interpreter.registers.cpu.get(inst.rs1)? as u32
                        ))
                    } else {
                        None
                    }
                }
                Self::CSRRWI_FUNCT3 => Some(CSOperation::Write(inst.rs1 as u32)),
                Self::CSRRSI_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Set(inst.rs1 as u32))
                    } else {
                        None
                    }
                }
                Self::CSRRCI_FUNCT3 => {
                    if inst.rs1 != 0 {
                        Some(CSOperation::Clear(inst.rs1 as u32))
                    } else {
                        None
                    }
                }
                _ => return Err(Error::InvalidInstruction(data)),
            };

            let res = interpreter
                .registers
                .control_status
                .operation(op, (inst.imm & 0b1111_1111_1111) as u16)?;

            if inst.rd_rs2 != 0 {
                let rd = interpreter.registers.cpu.get_mut(inst.rd_rs2)?;
                *rd = res as i32;
            }

            Ok(State::Running)
        };

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        format::{Format, TypeI},
        interpreter::{memory::SliceMemory, Config},
    };

    #[test]
    fn test_ebreak() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: 0x1,
            funct3: 0,
        };

        let result = SystemMiscMem::decode_execute(misc_mem.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(interpreter.program_counter, SystemMiscMem::SIZE as u32);
    }

    #[test]
    fn test_ecall() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(
            &mut memory,
            Config {
                ..Default::default()
            },
        )
        .unwrap();

        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::ECALL_IMM,
            funct3: SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(misc_mem.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Called));
        assert_eq!(interpreter.program_counter, SystemMiscMem::SIZE as u32);
    }

    #[test]
    fn test_wfi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::WFI_IMM,
            funct3: SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(misc_mem.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Waiting));
        assert_eq!(interpreter.program_counter, SystemMiscMem::SIZE as u32);
    }

    #[test]
    fn test_mret() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x341)
            .unwrap(); //mepc

        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::MRET_IMM,
            funct3: SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(misc_mem.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1234);
    }

    #[test]
    fn test_fencei() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::FENCEI_IMM,
            funct3: SystemMiscMem::EBREAK_ECALL_FENCEI_WFI_MRET_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(misc_mem.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, SystemMiscMem::SIZE as u32);
    }

    #[test]
    fn test_csrrw() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let csrrw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRW_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrw.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrs() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1230)), 0x342)
            .unwrap();

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let csrrs = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRS_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrs.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0x1230);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrc() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x342)
            .unwrap();

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1230;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1230;

        let csrrc = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRC_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrc.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0x1234);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x4
        );
    }

    #[test]
    fn test_csrrwi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;

        let csrrwi = TypeI {
            rd_rs2: 1,
            rs1: 1,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRWI_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrwi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x1
        );
    }

    #[test]
    fn test_csrrsi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1230)), 0x342)
            .unwrap();

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;

        let csrrsi = TypeI {
            rd_rs2: 1,
            rs1: 4,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRSI_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrsi.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0x1230);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x1234
        );
    }

    #[test]
    fn test_csrrci() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x342)
            .unwrap();

        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1230;

        let csrrci = TypeI {
            rd_rs2: 1,
            rs1: 4,
            imm: 0x342,
            funct3: SystemMiscMem::CSRRCI_FUNCT3,
        };

        let result = SystemMiscMem::decode_execute(csrrci.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(1).unwrap(), 0x1234);
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x342)
                .unwrap(),
            0x1230
        );
    }
}
