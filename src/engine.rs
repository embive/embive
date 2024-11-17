//! Engine Module

#[cfg(feature = "start_at_ram")]
use memory::RAM_OFFSET;

use crate::memory::Memory;
use crate::register::{Register, Registers};
use crate::error::EmbiveError;
use crate::instruction::decode_and_execute;

/// Number of syscall arguments
pub const SYSCALL_ARGS: usize = 6;

/// System call function signature
/// 
/// This function is called by the `ecall` instruction. Check [syscall(2)](https://man7.org/linux/man-pages/man2/syscall.2.html).
/// The following RISC-V registers are used for system calls:
/// - `a7`: System call number.
/// - `a0` to `a5`: System call arguments.
/// - `a0` and `a1`: Return values.
///
/// Arguments:
/// - `nr`: System call number.
/// - `args`: System call arguments, up to [`SYSCALL_ARGS`].
/// - `memory`: The virtual RISC-V engine memory.
///
/// Returns:
/// - `(i32, i32)`: Return values.
pub type SyscallFn = fn(nr: i32, args: [i32; SYSCALL_ARGS], memory: &mut Memory) -> (i32, i32);

/// Embive Engine Struct
#[derive(Debug)]
pub struct Engine<'a> {
    /// Program counter.
    pub(crate) pc: u32,
    /// CPU Registers.
    pub(crate) registers: Registers,
    /// System Memory (program + RAM).
    pub(crate) memory: Memory<'a>,
    /// System call function (Called by `ecall` instruction).
    syscall_fn: Option<SyscallFn>,
}

impl<'a> Engine<'a> {
    /// Create a new virtual RISC-V engine.
    ///
    /// Arguments:
    /// - `code`: Code buffer, `u8` slice.
    /// - `ram`: RAM buffer, mutable `u8` slice.
    /// - `syscall_fn`: Optional function to handle system calls.
    pub fn new(
        code: &'a [u8],
        ram: &'a mut [u8],
        syscall_fn: Option<SyscallFn>,
    ) -> Result<Engine<'a>, EmbiveError> {
        let memory = Memory::new(code, ram);

        Ok(Engine {
            pc: 0,
            registers: Registers::new(&memory),
            memory,
            syscall_fn,
        })
    }

    /// Reset the virtual RISC-V engine:
    /// - Program counter is reset to 0. (Or [`crate::memory::RAM_OFFSET`] if the `start_at_ram` feature is enabled).
    /// - Registers are reset to 0.
    /// - Stack pointer (x2) is set to the top of the stack.
    pub fn reset(&mut self) {
        #[cfg(feature = "start_at_ram")]
        {
            self.pc = RAM_OFFSET;
        }
        #[cfg(not(feature = "start_at_ram"))]
        {
            self.pc = 0;
        }

        self.registers.reset(&self.memory);
    }

    /// Run the virtual RISC-V engine from the start, until a halt instruction is reached.
    /// The engine is reset before execution (by implicitly calling [`Engine::reset`]).
    ///
    /// Returns:
    /// - `Ok`: Executed successfully (halt was reached).
    /// - `Err(EmbiveError)`: Failed to execute.
    pub fn run(&mut self) -> Result<(), EmbiveError> {
        self.reset();

        loop {
            // Step through the program
            if !self.step()? {
                // Halt execution
                return Ok(());
            }
        }
    }

    /// Run a single instruction from the virtual RISC-V engine.
    ///
    /// Returns:
    /// - `Ok(bool)`: Executed successfully.
    ///     - `True`: Should continue execution.
    ///     - `False`: Should halt.
    /// - `Err(EmbiveError)`: Failed to execute.
    #[inline]
    pub fn step(&mut self) -> Result<bool, EmbiveError> {
        // Fetch next instruction
        let data = self.fetch()?;

        // Decode and execute the instruction
        decode_and_execute(self, data)
    }

    /// Fetch the next instruction (raw) from the program counter.
    ///
    /// Returns:
    /// - `Ok(u32)`: The instruction (raw) that was fetched.
    /// - `Err(EmbiveError)`: The program counter is out of bounds.
    #[inline]
    pub fn fetch(&mut self) -> Result<u32, EmbiveError> {
        let data = self.memory.load::<4>(self.pc)?;
        Ok(u32::from_le_bytes(data))
    }

    /// Get a reference to the registers.
    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    /// Get a mutable reference to the registers.
    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// Get a reference to the memory.
    pub fn memory(&self) -> &Memory<'a> {
        &self.memory
    }

    /// Get a mutable reference to the memory.
    pub fn memory_mut(&mut self) -> &mut Memory<'a> {
        &mut self.memory
    }

    /// Get the program counter.
    pub fn pc(&self) -> u32 {
        self.pc
    }

    /// Set the program counter.
    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    /// Handle a system call.
    /// The system call function is called with the system call number and arguments.
    ///
    /// Returns:
    /// - `Ok(())`: Syscall executed.
    /// - `Err(EmbiveError)`: Failed to execute the system call function.
    ///     - System call function is not set.
    #[inline(always)]
    pub(crate) fn syscall(&mut self) -> Result<(), EmbiveError> {
        if let Some(syscall_fn) = self.syscall_fn {
            // Syscall Number
            let nr = self.registers.inner[Register::A7 as usize];

            // Syscall Arguments
            let args = [
                self.registers.inner[Register::A0 as usize],
                self.registers.inner[Register::A1 as usize],
                self.registers.inner[Register::A2 as usize],
                self.registers.inner[Register::A3 as usize],
                self.registers.inner[Register::A4 as usize],
                self.registers.inner[Register::A5 as usize],
            ];

            // Call the syscall function
            let (val, val2) = syscall_fn(nr, args, &mut self.memory);

            // Set return values
            self.registers.inner[Register::A0 as usize] = val;
            self.registers.inner[Register::A1 as usize] = val2;

            return Ok(());
        }

        // No syscall function set
        Err(EmbiveError::NoSyscallFunction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let mut engine = Engine::new(&[], &mut [], None).unwrap();
        engine.reset();

        #[cfg(feature = "start_at_ram")]
        {
            assert_eq!(engine.pc, RAM_OFFSET);
        }

        #[cfg(not(feature = "start_at_ram"))]
        {
            assert_eq!(engine.pc, 0);
        }

        assert_eq!(engine.registers.get(Register::Zero as usize).unwrap(), 0);
        assert_eq!(engine.registers.get(Register::Ra as usize).unwrap(), 0);
        assert_eq!(
            engine.registers.get(Register::Sp as usize).unwrap(),
            engine.memory().ram_end() as i32
        );

        for i in Register::Gp as usize..32 {
            assert_eq!(engine.registers.get(i).unwrap(), 0);
        }
    }
}
