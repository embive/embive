//! Control and Status Register Module
use crate::interpreter::{error::Error, EMBIVE_INTERRUPT_CODE};

/// Machine Status Register
const MSTATUS_ADDR: u16 = 0x300;
/// ISA and extensions supported.
const MISA_ADDR: u16 = 0x301;
/// Machine Interrupt Enable
const MIE_ADDR: u16 = 0x304;
/// Machine Trap Vector
const MTVEC_ADDR: u16 = 0x305;
/// Machine Status High Register
const MSTATUSH_ADDR: u16 = 0x310;
/// Inhibit machine counter/timer.
const MCOUNTINHIBIT_ADDR: u16 = 0x320;
/// Machine Scratch Register
const MSCRATCH_ADDR: u16 = 0x340;
/// Machine Exception Program Counter
const MEPC_ADDR: u16 = 0x341;
/// Machine Cause Register
const MCAUSE_ADDR: u16 = 0x342;
/// Machine Trap Value
const MTVAL_ADDR: u16 = 0x343;
/// Machine Interrupt Pending
const MIP_ADDR: u16 = 0x344;
/// Machine High Performance Event 31 High
const MHPMEVENT31H_ADDR: u16 = 0x33F;
/// Machine cycle counter.
const MCYCLE_ADDR: u16 = 0xB00;
/// Machine High Performance Counter 31 High
const MHPMCOUNTER31H_ADDR: u16 = 0xB9F;
/// Vendor ID
const MVENDORID_ADDR: u16 = 0xF11;
/// Pointer to configuration data structure
const MCONFIGPTR_ADDR: u16 = 0xF15;

/// Machine XLEN
const MXLEN: u32 = 32;
/// MXL for MXLEN = 32
const MXL_32: u32 = 0b01;
/// MISA A Extension
const MISA_A: u32 = 1 << 0;
/// MISA I Extension
const MISA_I: u32 = 1 << 8;
/// MISA M Extension
const MISA_M: u32 = 1 << 12;

/// MTVEC mode bits
const MTVEC_MODE: u32 = 0b11;

/// MEPC bit 0
const MEPC_BIT0: u32 = 0b1;

/// MSTATUS MIE bit
const MSTATUS_MIE: u8 = 0b1 << 3;
/// MSTATUS MPIE bit
const MSTATUS_MPIE: u8 = 0b1 << 7;
/// MSTATUS write mask
const MSTATUS_MASK: u8 = MSTATUS_MIE | MSTATUS_MPIE;

/// MCAUSE for Embive Custom Interrupt
const MCAUSE_MEI_CODE: u32 = EMBIVE_INTERRUPT_CODE;
/// MCAUSE interrupt bit
const MCAUSE_INTERRUPT: u32 = 0b1 << 31;

/// MIx (MIE and MIP) write mask for Embive Custom Interrupt
const MI_E_P_MASK: u32 = 0b1 << EMBIVE_INTERRUPT_CODE;

/// Control and Status Operation
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CSOperation {
    /// Write value to the register.
    Write(u32),
    /// Set Bits in the register.
    Set(u32),
    /// Clear Bits in the register.
    Clear(u32),
}

const fn get_misa() -> u32 {
    (MXL_32 << (MXLEN - 2)) | MISA_I | MISA_M | MISA_A
}

/// Control and Status Registers
/// Supported CSRs:
/// - MSTATUS (MIE, MPIE)
/// - MISA
/// - MIE (bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`])
/// - MTVEC (Direct mode only)
/// - MSCRATCH
/// - MEPC
/// - MCAUSE
/// - MTVAL
/// - MIP (bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`])
///
/// Ignored CSRs (read-only as 0):
/// - MSTATUSH
/// - MCOUNTINHIBIT..MHPMEVENT31
/// - MCYCLE..MHPMCOUNTER31
/// - MVENDORID..MCONFIGPTR
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct CSRegisters {
    /// Machine Trap Vector
    mtvec: u32,
    /// Machine Scratch Register
    mscratch: u32,
    /// Machine Exception Program Counter
    mepc: u32,
    /// Machine Cause Register
    mcause: u32,
    /// Machine Trap Value Register
    mtval: i32,
    /// Machine Interrupt Enable (bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`])
    mie_embive: bool,
    /// Machine Interrupt Pending (bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`])
    mip_embive: bool,
    /// Machine Status Register (MIE, MPIE)
    mstatus: u8,
}

impl CSRegisters {
    /// Execute a control and status register operation.
    ///
    /// Arguments:
    /// - `op`: The operation to execute.
    ///     - `None`: No operation, only read the register.
    /// - `addr`: The address of the register (from 0 to 4095).
    ///
    /// Returns:
    /// - `Ok(u32)`: The register value prior to the operation.
    /// - `Err(Error)`: The register address is invalid or not supported.
    #[inline]
    pub fn operation(&mut self, op: Option<CSOperation>, addr: u16) -> Result<u32, Error> {
        match addr {
            MSTATUS_ADDR => {
                let ret = self.mstatus as u32;
                self.mstatus = (execute_operation(op, ret) as u8) & MSTATUS_MASK;
                Ok(ret)
            }
            MISA_ADDR => Ok(get_misa()), // ISA and extensions supported
            MIE_ADDR => {
                let ret = (self.mie_embive as u32) << EMBIVE_INTERRUPT_CODE;
                self.mie_embive = (execute_operation(op, ret) & MI_E_P_MASK) != 0;
                Ok(ret)
            }
            MTVEC_ADDR => {
                let ret = self.mtvec;
                // We only support direct mode right now
                self.mtvec = execute_operation(op, ret) & !MTVEC_MODE;
                Ok(ret)
            }
            MSTATUSH_ADDR => Ok(0), // Ignore high mstatus
            MCOUNTINHIBIT_ADDR..=MHPMEVENT31H_ADDR => Ok(0), // Ignore counters
            MSCRATCH_ADDR => {
                let ret = self.mscratch;
                self.mscratch = execute_operation(op, ret);
                Ok(ret)
            }
            MEPC_ADDR => {
                let ret = self.mepc;
                // Bit 0 is always 0
                self.mepc = execute_operation(op, ret) & !MEPC_BIT0;
                Ok(ret)
            }
            MCAUSE_ADDR => {
                let ret = self.mcause;
                self.mcause = execute_operation(op, ret);
                Ok(ret)
            }
            MTVAL_ADDR => {
                let ret = self.mtval as u32;
                self.mtval = execute_operation(op, ret) as i32;
                Ok(ret)
            }
            MIP_ADDR => {
                let ret = (self.mip_embive as u32) << EMBIVE_INTERRUPT_CODE;
                self.mip_embive = (execute_operation(op, ret) & MI_E_P_MASK) != 0;
                Ok(ret)
            }
            MCYCLE_ADDR..=MHPMCOUNTER31H_ADDR => Ok(0), // Ignore counters
            MVENDORID_ADDR..=MCONFIGPTR_ADDR => Ok(0),  // IDs are always 0
            _ => Err(Error::InvalidCSRegister(addr)),
        }
    }

    /// Set the interrupt pending flag.
    /// Set `mip` bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`] to 1.
    ///
    /// Arguments:
    /// - `mtval`: The trap value.
    #[inline(always)]
    pub(crate) fn set_interrupt(&mut self) {
        // Set interrupt pending flag
        self.mip_embive = true;
    }

    /// Check if interrupt is enabled.
    /// Returns true if `mie` bit [`crate::interpreter::EMBIVE_INTERRUPT_CODE`] and `mstatus.MIE` are set.
    #[inline(always)]
    pub(crate) fn interrupt_enabled(&self) -> bool {
        self.mie_embive && (self.mstatus & MSTATUS_MIE) != 0
    }

    /// Trap Entry.
    /// This function triggers an interrupt trap.
    /// What it does:
    /// - Copy `mstatus.MIE` to `mstatus.MPIE` and then clear `mstatus.MIE`.
    /// - Set `mcause.MEI` to 1
    /// - Set `mcause.code` to 11
    /// - Copy the received program counter to `mepc`.
    /// - Copy the received value to `mtval`.
    /// - Update the program counter to the value in `mtvec`.
    ///
    /// Arguments:
    /// - `pc`: Mutable reference to the program counter.
    pub(crate) fn trap_entry(&mut self, pc: &mut u32, value: i32) {
        // Copy MIE to MPIE
        if (self.mstatus & MSTATUS_MIE) != 0 {
            self.mstatus |= MSTATUS_MPIE;
        } else {
            self.mstatus &= !MSTATUS_MPIE;
        }

        // Clear MIE
        self.mstatus &= !MSTATUS_MIE;

        // Set mcause
        self.mcause = MCAUSE_INTERRUPT | MCAUSE_MEI_CODE;

        // Copy PC to MEPC
        self.mepc = *pc;

        // Copy value to mtval
        self.mtval = value;

        // Update PC to mtvec
        *pc = self.mtvec & !MTVEC_MODE;
    }

    /// Trap Return.
    /// This function returns from an interrupt.
    /// What it does:
    /// - Restore `mstatus.MIE` from `mstatus.MPIE`.
    /// - Return the program counter from `mepc`.
    ///
    /// Returns:
    /// - `u32`: The program counter from `mepc`.
    pub(crate) fn trap_return(&mut self) -> u32 {
        // Copy MPIE to MIE
        if (self.mstatus & MSTATUS_MPIE) != 0 {
            self.mstatus |= MSTATUS_MIE;
        } else {
            self.mstatus &= !MSTATUS_MIE;
        }

        // Return the PC
        self.mepc
    }
}

#[inline]
fn execute_operation(op: Option<CSOperation>, value: u32) -> u32 {
    match op {
        Some(CSOperation::Write(val)) => val,
        Some(CSOperation::Set(val)) => value | val,
        Some(CSOperation::Clear(val)) => value & !val,
        None => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mstatus() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0x1898)), MSTATUS_ADDR),
            Ok(0)
        );
        assert_eq!(
            cs.operation(None, MSTATUS_ADDR),
            Ok(0x1898 & MSTATUS_MASK as u32)
        );
    }

    #[test]
    fn test_misa() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0x1898)), MISA_ADDR),
            Ok(get_misa())
        );
        assert_eq!(cs.operation(None, MISA_ADDR), Ok(get_misa()));
    }

    #[test]
    fn test_mie() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0x1810)), MIE_ADDR),
            Ok(0)
        );
        assert_eq!(cs.operation(None, MIE_ADDR), Ok(0x1810 & MI_E_P_MASK));
    }

    #[test]
    fn test_mtvec() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0x12FF)), MTVEC_ADDR),
            Ok(0)
        );
        assert_eq!(cs.operation(None, MTVEC_ADDR), Ok(0x12FF & !MTVEC_MODE));
    }

    #[test]
    fn test_mscratch() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0xFFFF)), MSCRATCH_ADDR),
            Ok(0)
        );
        assert_eq!(cs.operation(None, MSCRATCH_ADDR), Ok(0xFFFF));
    }

    #[test]
    fn test_mepc() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0x1231)), MEPC_ADDR),
            Ok(0)
        );
        assert_eq!(cs.operation(None, MEPC_ADDR), Ok(0x1231 & !MEPC_BIT0));
    }

    #[test]
    fn test_mcause() {
        let mut cs = CSRegisters::default();

        assert_eq!(
            cs.operation(Some(CSOperation::Write(0xFFFF)), MCAUSE_ADDR),
            Ok(0)
        );
        assert_eq!(cs.operation(None, MCAUSE_ADDR), Ok(0xFFFF));
    }

    #[test]
    fn test_mip() {
        let mut cs = CSRegisters::default();

        assert_eq!(cs.operation(None, MIP_ADDR), Ok(0));

        // set interrupt
        cs.set_interrupt();
        assert_eq!(cs.operation(None, MIP_ADDR), Ok(MI_E_P_MASK));
    }
}
