//! Control and Status Register Module
use crate::error::Error;

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
/// Machine High Performance Event 31
const MHPMEVENT31_ADDR: u16 = 0x33F;
/// Machine cycle counter.
const MCYCLE_ADDR: u16 = 0xB00;
/// Machine High Performance Counter 31
const MHPMCOUNTER31_ADDR: u16 = 0xB9F;
/// Vendor ID
const MVENDORID_ADDR: u16 = 0xF11;
/// Pointer to configuration data structure
const MCONFIGPTR_ADDR: u16 = 0xF15;

/// Machine XLEN
const MXLEN: u32 = 32;
/// MXL for MXLEN = 32
const MXL_32: u32 = 0b01;
/// MISA A Extension
#[cfg(feature = "a_extension")]
const MISA_A: u32 = 1 << 0;
/// MISA I Extension
const MISA_I: u32 = 1 << 8;
/// MISA M Extension
#[cfg(feature = "m_extension")]
const MISA_M: u32 = 1 << 12;

/// MTVEC mode bits
const MTVEC_MODE: u32 = 0b11;

/// MEPC bit 0
const MEPC_BIT0: u32 = 0b1;

/// MSTATUS MIE bit
const MSTATUS_MIE: u32 = 0b1 << 3;
/// MSTATUS MPIE bit
const MSTATUS_MPIE: u32 = 0b1 << 7;
/// MSTATUS write mask
const MSTATUS_MASK: u32 = MSTATUS_MIE | MSTATUS_MPIE;

/// MCAUSE Machine External Interrupt code
const MCAUSE_MEI_CODE: u32 = 11;
/// MCAUSE interrupt bit
const MCAUSE_INTERRUPT: u32 = 0b1 << 31;

/// MIE write mask (MEIE)
const MIE_MASK: u32 = 0b1 << 11;

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
    let mut misa = MXL_32 << (MXLEN - 2);

    #[cfg(feature = "a_extension")]
    {
        misa |= MISA_A;
    }

    misa |= MISA_I;

    #[cfg(feature = "m_extension")]
    {
        misa |= MISA_M;
    }

    misa
}

/// Control and Status Registers
/// Supported CSRs:
/// - MSTATUS (MIE, MPIE)
/// - MISA (Read-only)
/// - MIE (MEIE)
/// - MTVEC (Direct mode only)
/// - MEPC
/// - MCAUSE
///
/// Ignored CSRs (read-only as 0):
/// - MSTATUSH
/// - MCOUNTINHIBIT..MHPMEVENT31
/// - MSCRATCH
/// - MTVAL
/// - MIP
/// - MCYCLE..MHPMCOUNTER31
/// - MVENDORID..MCONFIGPTR
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct CSRegisters {
    /// Machine Status Register
    mstatus: u32,
    /// Machine Interrupt Enable
    mie: u32,
    /// Machine Trap Vector
    mtvec: u32,
    /// Machine Exception Program Counter
    mepc: u32,
    /// Machine Cause Register
    mcause: u32,
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
                let ret = self.mstatus;
                self.mstatus = execute_operation(op, self.mstatus) & MSTATUS_MASK;
                Ok(ret)
            }
            MISA_ADDR => Ok(get_misa()), // ISA and extensions supported
            MIE_ADDR => {
                let ret = self.mie;
                self.mie = execute_operation(op, self.mie) & MIE_MASK;
                Ok(ret)
            }
            MTVEC_ADDR => {
                let ret = self.mtvec;
                // We only support direct mode right now
                self.mtvec = execute_operation(op, self.mtvec) & !MTVEC_MODE;
                Ok(ret)
            }
            MSTATUSH_ADDR => Ok(0), // Ignore high mstatus
            MCOUNTINHIBIT_ADDR..=MHPMEVENT31_ADDR => Ok(0), // Ignore counters
            MSCRATCH_ADDR => Ok(0), // Ignore mscratch
            MEPC_ADDR => {
                let ret = self.mepc;
                // Bit 0 is always 0
                self.mepc = execute_operation(op, self.mepc) & !MEPC_BIT0;
                Ok(ret)
            }
            MCAUSE_ADDR => {
                let ret = self.mcause;
                self.mcause = execute_operation(op, self.mcause);
                Ok(ret)
            }
            MTVAL_ADDR => Ok(0),                       // Ignore trap value
            MIP_ADDR => Ok(0),                         // Ignore pending interrupts
            MCYCLE_ADDR..=MHPMCOUNTER31_ADDR => Ok(0), // Ignore counters
            MVENDORID_ADDR..=MCONFIGPTR_ADDR => Ok(0), // IDs are always 0
            _ => Err(Error::InvalidCSRegister),
        }
    }

    /// Trap Entry.
    /// This function triggers an interrupt/callback.
    /// What it does:
    /// - Copy `mstatus.MIE` to `mstatus.MPIE`, then set `mstatus.MIE` to 0.
    /// - Set `mcause` as a machine external interrupt.
    /// - Copy the received `PC` to `mepc`.
    /// - Return the new `PC` value.
    ///
    /// Arguments:
    /// - `pc`: The current program counter.
    ///
    /// Returns:
    /// - `Ok(u32)`: The new program counter.
    /// - `Err(Error)`: Failed to handle the interrupt.
    pub(crate) fn trap_entry(&mut self, pc: u32) -> Result<u32, Error> {
        // Check if interrupts are enabled (mstatus.MIE or mie.MEIP)
        if (self.mstatus & MSTATUS_MIE) == 0 || (self.mie & MIE_MASK) == 0 {
            return Err(Error::CallbackNotEnabled);
        }

        // Set MPIE and clear MIE
        self.mstatus = (self.mstatus & !MSTATUS_MIE) | MSTATUS_MPIE;

        // Set mcause
        self.mcause = MCAUSE_INTERRUPT | MCAUSE_MEI_CODE;

        // Copy PC to MEPC
        self.mepc = pc;

        // Return the new PC
        Ok(self.mtvec)
    }

    /// Trap Return.
    /// This function returns from an interrupt/callback.
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
        assert_eq!(cs.operation(None, MSTATUS_ADDR), Ok(0x1898 & MSTATUS_MASK));
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
        assert_eq!(cs.operation(None, MIE_ADDR), Ok(0x1810 & MIE_MASK));
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
}
