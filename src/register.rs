//! Register Module

use crate::error::EmbiveError;
use crate::memory::Memory;

/// Number of registers in embive
pub const REGISTER_COUNT: usize = 32;

/// Embive Register Enum
#[repr(usize)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum Register {
    /// x0 register, hardwired to 0 (read-only).
    Zero = 0,
    /// x1 register, return address.
    Ra = 1,
    /// x2 register, stack pointer.
    Sp = 2,
    /// x3 register, global pointer.
    Gp = 3,
    /// x4 register, thread pointer.
    Tp = 4,
    /// x5 register, temporary.
    T0 = 5,
    /// x6 register, temporary.
    T1 = 6,
    /// x7 register, temporary.
    T2 = 7,
    /// x8 register, saved.
    S0 = 8,
    /// x9 register, saved.
    S1 = 9,
    /// x10 register, function argument/return.
    A0 = 10,
    /// x11 register, function argument/return.
    A1 = 11,
    /// x12 register, function argument.
    A2 = 12,
    /// x13 register, function argument.
    A3 = 13,
    /// x14 register, function argument.
    A4 = 14,
    /// x15 register, function argument.
    A5 = 15,
    /// x16 register, function argument.
    A6 = 16,
    /// x17 register, function argument.
    A7 = 17,
    /// x18 register, saved.
    S2 = 18,
    /// x19 register, saved.
    S3 = 19,
    /// x20 register, saved.
    S4 = 20,
    /// x21 register, saved.
    S5 = 21,
    /// x22 register, saved.
    S6 = 22,
    /// x23 register, saved.
    S7 = 23,
    /// x24 register, saved.
    S8 = 24,
    /// x25 register, saved.
    S9 = 25,
    /// x26 register, saved.
    S10 = 26,
    /// x27 register, saved.
    S11 = 27,
    /// x28 register, temporary.
    T3 = 28,
    /// x29 register, temporary.
    T4 = 29,
    /// x30 register, temporary.
    T5 = 30,
    /// x31 register, temporary.
    T6 = 31,
}

/// Embive Registers
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Registers {
    pub(crate) inner: [i32; REGISTER_COUNT as usize],
}

impl Registers {
    /// Create a new set of registers.
    /// The stack pointer (x2) is set to the end of memory, all other registers are set to 0.
    pub(crate) fn new(memory: &Memory<'_>) -> Registers {
        Registers {
            inner: initial_registers(memory),
        }
    }

    /// Reset the registers to their initial state.
    /// The stack pointer (x2) is set to the end of memory, all other registers are set to 0.
    pub(crate) fn reset(&mut self, memory: &Memory<'_>) {
        self.inner = initial_registers(memory);
    }

    /// Get a register.
    ///
    /// Arguments:
    /// - `index`: The index of the register (from `0` to [`REGISTER_COUNT`]).
    ///
    /// Returns:
    /// - `Ok(i32)`: The value of the register.
    /// - `Err(EmbiveError)`: The register index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Result<i32, EmbiveError> {
        if index >= REGISTER_COUNT {
            return Err(EmbiveError::InvalidRegister);
        }

        Ok(self.inner[index])
    }

    /// Get a mutable reference to a register.
    ///
    /// Arguments:
    /// - `index`: The index of the register (from `0` to [`REGISTER_COUNT`]).
    ///     - Register `0` (`Register::Zero`) should be read-only, we ignore it for performance reasons.
    ///
    /// Returns:
    /// - `Ok(&mut i32)`: Mutable reference to the register.
    /// - `Err(EmbiveError)`: The register index is out of bounds.
    #[inline]
    pub(crate) fn get_mut(&mut self, index: usize) -> Result<&mut i32, EmbiveError> {
        if index >= REGISTER_COUNT {
            return Err(EmbiveError::InvalidRegister);
        }

        Ok(&mut self.inner[index])
    }
}

/// Get the initial registers state.
/// The stack pointer (`x2`) is set to the end of memory, all other registers are set to `0`.
fn initial_registers(memory: &Memory<'_>) -> [i32; REGISTER_COUNT as usize] {
    let mut registers = [0; REGISTER_COUNT as usize];

    // Set the stack pointer to the top of the stack
    registers[Register::Sp as u8 as usize] = memory.ram_end() as i32;

    registers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_register() {
        let registers = Registers {
            inner: [0; REGISTER_COUNT as usize],
        };

        assert_eq!(registers.get(0), Ok(0));
        assert_eq!(registers.get(REGISTER_COUNT as usize - 1), Ok(0));
    }

    #[test]
    fn get_register_out_of_bounds() {
        let registers = Registers {
            inner: [0; REGISTER_COUNT as usize],
        };

        assert_eq!(
            registers.get(REGISTER_COUNT as usize),
            Err(EmbiveError::InvalidRegister)
        );
    }
}
