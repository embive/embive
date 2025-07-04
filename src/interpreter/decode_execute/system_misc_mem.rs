use crate::instruction::embive::InstructionImpl;
use crate::instruction::embive::SystemMiscMem;
use crate::interpreter::{memory::Memory, registers::CSOperation, Error, Interpreter, State};

use super::Execute;

impl<M: Memory> Execute<M> for SystemMiscMem {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let ret = if self.0.func == Self::MISC_FUNC {
            match self.0.imm {
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
                _ => return Err(Error::InvalidInstruction(interpreter.program_counter)),
            }
        } else {
            let op = match self.0.func {
                Self::CSRRW_FUNC => Some(CSOperation::Write(
                    interpreter.registers.cpu.get(self.0.rs1)? as u32,
                )),
                Self::CSRRS_FUNC => {
                    if self.0.rs1 != 0 {
                        Some(CSOperation::Set(
                            interpreter.registers.cpu.get(self.0.rs1)? as u32
                        ))
                    } else {
                        None
                    }
                }
                Self::CSRRC_FUNC => {
                    if self.0.rs1 != 0 {
                        Some(CSOperation::Clear(
                            interpreter.registers.cpu.get(self.0.rs1)? as u32,
                        ))
                    } else {
                        None
                    }
                }
                Self::CSRRWI_FUNC => Some(CSOperation::Write(self.0.rs1 as u32)),
                Self::CSRRSI_FUNC => {
                    if self.0.rs1 != 0 {
                        Some(CSOperation::Set(self.0.rs1 as u32))
                    } else {
                        None
                    }
                }
                Self::CSRRCI_FUNC => {
                    if self.0.rs1 != 0 {
                        Some(CSOperation::Clear(self.0.rs1 as u32))
                    } else {
                        None
                    }
                }
                _ => return Err(Error::InvalidInstruction(interpreter.program_counter)),
            };

            let res = interpreter
                .registers
                .control_status
                .operation(op, (self.0.imm & 0b1111_1111_1111) as u16)?;

            if self.0.rd_rs2 != 0 {
                let rd = interpreter.registers.cpu.get_mut(self.0.rd_rs2)?;
                *rd = res as i32;
            }

            Ok(State::Running)
        };

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        format::{Format, TypeI},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    #[test]
    fn test_ebreak() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: 0x1,
            func: 0,
        };

        let result = SystemMiscMem::decode(misc_mem.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(interpreter.program_counter, SystemMiscMem::size() as u32);
    }

    #[test]
    fn test_ecall() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let mut interpreter = Interpreter::new(&mut memory, 0);

        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::ECALL_IMM,
            func: SystemMiscMem::MISC_FUNC,
        };

        let result = SystemMiscMem::decode(misc_mem.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Called));
        assert_eq!(interpreter.program_counter, SystemMiscMem::size() as u32);
    }

    #[test]
    fn test_wfi() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::WFI_IMM,
            func: SystemMiscMem::MISC_FUNC,
        };

        let result = SystemMiscMem::decode(misc_mem.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Waiting));
        assert_eq!(interpreter.program_counter, SystemMiscMem::size() as u32);
    }

    #[test]
    fn test_mret() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter
            .registers
            .control_status
            .operation(Some(CSOperation::Write(0x1234)), 0x341)
            .unwrap(); //mepc

        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::MRET_IMM,
            func: SystemMiscMem::MISC_FUNC,
        };

        let result = SystemMiscMem::decode(misc_mem.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 0x1234);
    }

    #[test]
    fn test_fencei() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let misc_mem = TypeI {
            rd_rs2: 0,
            rs1: 0,
            imm: SystemMiscMem::FENCEI_IMM,
            func: SystemMiscMem::MISC_FUNC,
        };

        let result = SystemMiscMem::decode(misc_mem.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, SystemMiscMem::size() as u32);
    }

    #[test]
    fn test_csrrw() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;
        *interpreter.registers.cpu.get_mut(2).unwrap() = 0x1234;

        let csrrw = TypeI {
            rd_rs2: 1,
            rs1: 2,
            imm: 0x342,
            func: SystemMiscMem::CSRRW_FUNC,
        };

        let result = SystemMiscMem::decode(csrrw.to_embive()).execute(&mut interpreter);
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
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
            func: SystemMiscMem::CSRRS_FUNC,
        };

        let result = SystemMiscMem::decode(csrrs.to_embive()).execute(&mut interpreter);
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
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
            func: SystemMiscMem::CSRRC_FUNC,
        };

        let result = SystemMiscMem::decode(csrrc.to_embive()).execute(&mut interpreter);
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
        *interpreter.registers.cpu.get_mut(1).unwrap() = 0x1234;

        let csrrwi = TypeI {
            rd_rs2: 1,
            rs1: 1,
            imm: 0x342,
            func: SystemMiscMem::CSRRWI_FUNC,
        };

        let result = SystemMiscMem::decode(csrrwi.to_embive()).execute(&mut interpreter);
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
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
            func: SystemMiscMem::CSRRSI_FUNC,
        };

        let result = SystemMiscMem::decode(csrrsi.to_embive()).execute(&mut interpreter);
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
        let mut interpreter = Interpreter::new(&mut memory, 0);
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
            func: SystemMiscMem::CSRRCI_FUNC,
        };

        let result = SystemMiscMem::decode(csrrci.to_embive()).execute(&mut interpreter);
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
