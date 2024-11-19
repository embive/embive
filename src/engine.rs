//! Engine Module

use crate::error::EmbiveError;
use crate::instruction::decode_and_execute;
use crate::memory::Memory;
use crate::register::{Register, Registers};

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
/// - `memory`: Engine memory.
///
/// Returns:
/// - `Result<i32, i32>`: value (`a1`), error (`a0`).
pub type SyscallFn =
    fn(nr: i32, args: &[i32; SYSCALL_ARGS], memory: &mut Memory) -> Result<i32, i32>;

/// Embive Engine Configuration Struct
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Config {
    /// System call function (Called by `ecall` instruction).
    pub syscall_fn: Option<SyscallFn>,
    /// Instruction limit. Yield when the limit is reached (0 = No limit).
    #[cfg(feature = "instruction_limit")]
    pub instruction_limit: u32,
}

/// Embive Engine Struct
#[derive(Debug)]
pub struct Engine<'a> {
    /// Program counter.
    pub(crate) program_counter: u32,
    /// CPU Registers.
    pub(crate) registers: Registers,
    /// System Memory (program + RAM).
    pub(crate) memory: Memory<'a>,
    /// Engine Configuration.
    config: Config,
}

impl<'a> Engine<'a> {
    /// Create a new embive engine.
    ///
    /// Arguments:
    /// - `code`: Code buffer, `u8` slice.
    /// - `ram`: RAM buffer, mutable `u8` slice.
    /// - `config`: Engine configuration.
    pub fn new(
        code: &'a [u8],
        ram: &'a mut [u8],
        config: Config,
    ) -> Result<Engine<'a>, EmbiveError> {
        let memory = Memory::new(code, ram);

        // Create the engine
        Ok(Engine {
            program_counter: 0,
            registers: Registers::new(&memory),
            memory,
            config,
        })
    }

    /// Reset the engine:
    /// - Program counter is reset to 0.
    /// - Registers are reset to 0.
    /// - Stack pointer (x2) is set to the top of the stack.
    /// - Instruction counter is reset to 0 (if the `instruction_limit` feature is enabled).
    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.registers.reset(&self.memory);
    }

    /// Run the engine
    /// If the `instruction_limit` feature is enabled, the engine will yield when the limit is reached.
    ///
    /// Returns:
    /// - `Ok(bool)`: Success, returns if should continue:
    ///     - `True`: Continue running (yielded, call `run` again).
    ///     - `False`: Stop running (halted, call `reset` prior to running again).
    /// - `Err(EmbiveError)`: Failed to run.
    pub fn run(&mut self) -> Result<bool, EmbiveError> {
        #[cfg(feature = "instruction_limit")]
        {
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
    /// - `Err(EmbiveError)`: Failed to execute.
    #[inline]
    pub fn step(&mut self) -> Result<bool, EmbiveError> {
        // Fetch next instruction
        let data = self.fetch()?;

        // Decode and execute the instruction
        let ret = decode_and_execute(self, data)?;

        Ok(ret)
    }

    /// Fetch the next instruction (raw) from the program counter.
    ///
    /// Returns:
    /// - `Ok(u32)`: The instruction (raw) that was fetched.
    /// - `Err(EmbiveError)`: The program counter is out of bounds.
    #[inline]
    pub fn fetch(&mut self) -> Result<u32, EmbiveError> {
        let data = self.memory.load::<4>(self.program_counter)?;
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
    pub fn program_counter(&self) -> u32 {
        self.program_counter
    }

    /// Set the program counter.
    pub fn set_program_counter(&mut self, program_counter: u32) {
        self.program_counter = program_counter;
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
        if let Some(syscall_fn) = self.config.syscall_fn {
            // Syscall Number
            let nr = self.registers.inner[Register::A7 as usize];

            // Syscall Arguments
            let args = self.registers.inner[Register::A0 as usize..]
                .first_chunk()
                // Unwrap is safe because the slice is guaranteed to have more than SYSCALL_ARGS elements.
                .unwrap();

            // Call the syscall function
            match syscall_fn(nr, args, &mut self.memory) {
                Ok(value) => {
                    // Clear error code
                    self.registers.inner[Register::A0 as usize] = 0;

                    // Set return value
                    self.registers.inner[Register::A1 as usize] = value;
                }
                Err(error) => {
                    // Set error code
                    self.registers.inner[Register::A0 as usize] = error;
                                        
                    // Clear return value
                    self.registers.inner[Register::A1 as usize] = 0;
                }
            }

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
        let mut engine = Engine::new(&[], &mut [], Default::default()).unwrap();
        engine.reset();

        assert_eq!(engine.program_counter, 0);

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

    #[cfg(feature = "instruction_limit")]
    #[test]
    fn test_instruction_limit() {
        let code = &[
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];

        let mut engine = Engine::new(
            code,
            &mut [],
            Config {
                instruction_limit: 2,
                ..Default::default()
            },
        )
        .unwrap();

        // Force program counter to 0x0 (bypass start_from_ram feature)
        engine.program_counter = 0;

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(true));
        assert_eq!(engine.program_counter, 4 * 2);

        // Run the engine again
        let result = engine.run();
        assert_eq!(result, Ok(false));
        assert_eq!(engine.program_counter, 4 * 4);
    }

    #[cfg(feature = "instruction_limit")]
    #[test]
    fn test_instruction_limit_zero() {
        let code = &[
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];

        let mut engine = Engine::new(
            code,
            &mut [],
            Config {
                instruction_limit: 0,
                ..Default::default()
            },
        )
        .unwrap();

        // Force program counter to 0x0 (bypass start_from_ram feature)
        engine.program_counter = 0;

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(false));
        assert_eq!(engine.program_counter, 4 * 4);
    }
}
