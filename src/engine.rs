//! Engine Module

use crate::error::Error;
use crate::instruction::decode_execute;
use crate::memory::Memory;
use crate::registers::{CPURegister, Registers};

/// Number of syscall arguments
pub const SYSCALL_ARGS: usize = 7;

/// System call function signature
///
/// This function is called by the `ecall` instruction.
/// The following registers are used:
/// - `a7`: Syscall number.
/// - `a0` to `a6`: Arguments.
/// - `a0`: Return error code.
/// - `a1`: Return value.
///
/// Arguments:
/// - `nr`: Syscall number (`a7`).
/// - `args`: Arguments (`a0` to `a6`).
/// - `memory`: System Memory (code + RAM).
///
/// Returns:
/// - `Result<i32, i32>`: value (`a1`), error (`a0`).
pub type SyscallFn<M> = fn(nr: i32, args: &[i32; SYSCALL_ARGS], memory: &mut M) -> Result<i32, i32>;

/// Embive Engine Configuration Struct
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub struct Config<M: Memory> {
    /// System call function (Called by `ecall` instruction).
    pub syscall_fn: Option<SyscallFn<M>>,
    /// Instruction limit. Yield when the limit is reached (0 = No limit).
    pub instruction_limit: u32,
}

impl<M: Memory> Config<M> {
    /// Set the system call function and return the configuration.
    ///
    /// Arguments:
    /// - `syscall_fn`: Optional system call function.
    pub fn with_syscall_fn(mut self, syscall_fn: Option<SyscallFn<M>>) -> Self {
        self.syscall_fn = syscall_fn;
        self
    }

    /// Set the instruction limit and return the configuration.
    ///
    /// Arguments:
    /// - `instruction_limit`: Instruction limit (0 = No limit).
    pub fn with_instruction_limit(mut self, instruction_limit: u32) -> Self {
        self.instruction_limit = instruction_limit;
        self
    }
}

impl<M: Memory> Default for Config<M> {
    fn default() -> Self {
        Config {
            syscall_fn: None,
            instruction_limit: 0,
        }
    }
}

/// Embive Engine Struct
#[derive(Debug)]
#[non_exhaustive]
pub struct Engine<'a, M: Memory> {
    /// Program Counter.
    pub program_counter: u32,
    /// CPU Registers.
    pub registers: Registers,
    /// System Memory (code + RAM).
    pub memory: &'a mut M,
    /// Engine Configuration.
    pub config: Config<M>,
    /// Memory reservation for atomic operations (addr, value).
    #[cfg(feature = "a_extension")]
    pub(crate) memory_reservation: Option<(u32, i32)>,
}

impl<'a, M: Memory> Engine<'a, M> {
    /// Create a new engine.
    ///
    /// Arguments:
    /// - `memory`: System memory (code + RAM).
    /// - `config`: Engine configuration.
    pub fn new(memory: &'a mut M, config: Config<M>) -> Result<Self, Error> {
        // Create the engine
        Ok(Engine {
            program_counter: 0,
            registers: Default::default(),
            memory,
            config,
            #[cfg(feature = "a_extension")]
            memory_reservation: None,
        })
    }

    /// Reset the engine:
    /// - Program counter is reset to 0.
    /// - CPU Registers are reset to 0.
    /// - Memory reservation is cleared.
    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.registers = Default::default();
        #[cfg(feature = "a_extension")]
        {
            self.memory_reservation = None;
        }
    }

    /// Run the engine
    /// If configured, the engine will yield when the instruction limit is reached.
    ///
    /// Returns:
    /// - `Ok(bool)`: Success, returns if should continue:
    ///     - `True`: Continue running (yielded, call `run` again).
    ///     - `False`: Stop running (halted, call `reset` prior to running again).
    /// - `Err(Error)`: Failed to run.
    pub fn run(&mut self) -> Result<bool, Error> {
        // Check if there is an instruction limit
        if self.config.instruction_limit > 0 {
            // Run the engine with an instruction limit
            for _ in 0..self.config.instruction_limit {
                // Step through the program
                if !self.step()? {
                    // Stop running
                    return Ok(false);
                }
            }

            // Yield
            return Ok(true);
        }

        // No instruction limit
        loop {
            // Step through the program
            if !self.step()? {
                // Stop running
                return Ok(false);
            }
        }
    }

    /// Step through a single instruction from the current program counter.
    ///
    /// Returns:
    /// - `Ok(bool)`: Success, returns if should continue:
    ///     - `True`: Should continue.
    ///     - `False`: Should stop (halted).
    /// - `Err(Error)`: Failed to execute.
    #[inline]
    pub fn step(&mut self) -> Result<bool, Error> {
        // Fetch next instruction
        let data = self.fetch()?;

        // Decode and execute the instruction
        let ret = decode_execute(self, data)?;

        Ok(ret)
    }

    /// Fetch the next instruction (raw) from the program counter.
    ///
    /// Returns:
    /// - `Ok(u32)`: The instruction (raw) that was fetched.
    /// - `Err(Error)`: The program counter is out of bounds.
    #[inline]
    pub fn fetch(&mut self) -> Result<u32, Error> {
        let data = self.memory.load::<4>(self.program_counter)?;
        Ok(u32::from_le_bytes(data))
    }

    /// Handle a system call.
    /// The system call function is called with the system call number and arguments.
    ///
    /// Returns:
    /// - `Ok(())`: Syscall executed.
    /// - `Err(Error)`: Failed to execute the system call function.
    ///     - System call function is not set.
    #[inline(always)]
    pub(crate) fn syscall(&mut self) -> Result<(), Error> {
        if let Some(syscall_fn) = self.config.syscall_fn {
            // Syscall Number
            let nr = self.registers.cpu.inner[CPURegister::A7 as usize];

            // Syscall Arguments
            let args = self.registers.cpu.inner[CPURegister::A0 as usize..]
                .first_chunk()
                // Unwrap is safe because the slice is guaranteed to have more than SYSCALL_ARGS elements.
                .unwrap();

            // Call the syscall function
            match syscall_fn(nr, args, self.memory) {
                Ok(value) => {
                    // Clear error code
                    self.registers.cpu.inner[CPURegister::A0 as usize] = 0;

                    // Set return value
                    self.registers.cpu.inner[CPURegister::A1 as usize] = value;
                }
                Err(error) => {
                    // Set error code
                    self.registers.cpu.inner[CPURegister::A0 as usize] = error;

                    // Clear return value
                    self.registers.cpu.inner[CPURegister::A1 as usize] = 0;
                }
            }

            return Ok(());
        }

        // No syscall function set
        Err(Error::NoSyscallFunction)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::SliceMemory;

    use super::*;

    #[test]
    fn test_reset() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();
        engine.reset();

        assert_eq!(engine.program_counter, 0);
    }

    #[test]
    fn test_instruction_limit() {
        let code = &[
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];

        let mut memory = SliceMemory::new(code, &mut []);
        let mut engine = Engine::new(
            &mut memory,
            Config {
                instruction_limit: 2,
                ..Default::default()
            },
        )
        .unwrap();

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 4 * 2);

        // Run the engine again
        let result = engine.run();
        assert_eq!(result, Ok(false));
        assert_eq!(engine.program_counter, 4 * 4);
    }

    #[test]
    fn test_instruction_limit_zero() {
        let code = &[
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];

        let mut memory = SliceMemory::new(code, &mut []);
        let mut engine = Engine::new(
            &mut memory,
            Config {
                instruction_limit: 0,
                ..Default::default()
            },
        )
        .unwrap();

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(false));
        assert_eq!(engine.program_counter, 4 * 4);
    }
}
