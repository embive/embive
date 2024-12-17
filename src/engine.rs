//! Engine Module
mod config;
mod state;
mod syscall;

use crate::error::Error;
use crate::instruction::decode_execute;
use crate::memory::Memory;
use crate::registers::{CPURegister, Registers};

#[doc(inline)]
pub use config::Config;
#[doc(inline)]
pub use state::EngineState;
#[doc(inline)]
pub use syscall::{SyscallFn, SYSCALL_ARGS};

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
    pub fn run(&mut self) -> Result<EngineState, Error> {
        // Check if there is an instruction limit
        if self.config.instruction_limit > 0 {
            // Run the engine with an instruction limit
            for _ in 0..self.config.instruction_limit {
                // Step through the program
                let state = self.step()?;

                if state != EngineState::Running {
                    // Stop running
                    return Ok(state);
                }
            }

            // Yield after the instruction limit (still running)
            return Ok(EngineState::Running);
        }

        // No instruction limit
        loop {
            // Step through the program
            let state = self.step()?;

            if state != EngineState::Running {
                // Stop running
                return Ok(state);
            }
        }
    }

    /// Step through a single instruction from the current program counter.
    ///
    /// Returns:
    /// - `Ok(EngineState)`: Success, current engine state.
    /// - `Err(Error)`: Failed to execute.
    #[inline(always)]
    pub fn step(&mut self) -> Result<EngineState, Error> {
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
    #[inline(always)]
    pub fn fetch(&mut self) -> Result<u32, Error> {
        let data = self.memory.load::<4>(self.program_counter)?;
        Ok(u32::from_le_bytes(data))
    }

    /// Set engine to the callback/interrupt configured by the interpreted code.
    /// This call does not execute any interpreted code, [`Engine::run`] should be called after.
    ///
    /// Interrupts are enabled by setting CSRs `mstatus.MIE` and `mie.MEIP`,
    /// as well as configuring `mtvec` with a valid address.
    ///
    /// Returns:
    /// - `Ok(())`: Success, engine is set to the callback/interrupt.
    /// - `Err(Error)`: Callback not enabled by interpreted code.
    pub fn callback(&mut self) -> Result<(), Error> {
        self.program_counter = self
            .registers
            .control_status
            .trap_entry(self.program_counter)?;

        Ok(())
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
        assert_eq!(result, Ok(EngineState::Running));
        assert_eq!(engine.program_counter, 4 * 2);

        // Run the engine again
        let result = engine.run();
        assert_eq!(result, Ok(EngineState::Halted));
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
        assert_eq!(result, Ok(EngineState::Halted));
        assert_eq!(engine.program_counter, 4 * 4);
    }

    #[test]
    fn test_callback() {
        let code = &[
            0x93, 0x00, 0x80, 0x00, // li   ra, 8
            0xf3, 0x90, 0x00, 0x30, // csrrw ra, mstatus, ra
            0x93, 0x00, 0x00, 0x80, // li   ra, -2048
            0xf3, 0x90, 0x40, 0x30, // csrrw ra, mie, ra
            0x93, 0x00, 0x80, 0x02, // li   ra, 40
            0xf3, 0x90, 0x50, 0x30, // csrrw ra, mtvec, ra
            0x13, 0x01, 0x70, 0x03, // li   sp, 55
            0x73, 0x00, 0x50, 0x10, // wfi
            0x93, 0x01, 0x70, 0x03, // li   gp, 55
            0x73, 0x00, 0x10, 0x00, // ebreak
            0x13, 0x01, 0x60, 0x01, // li   sp, 22
            0x73, 0x00, 0x20, 0x30, // mret
        ];

        let mut memory = SliceMemory::new(code, &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(EngineState::Waiting));
        assert_eq!(
            engine.registers.cpu.get(CPURegister::SP as usize).unwrap(),
            55
        );

        // Callback
        let result = engine.callback();
        assert_eq!(result, Ok(()));
        assert_eq!(engine.program_counter, 40);

        // Run the engine again
        let result = engine.run();
        assert_eq!(result, Ok(EngineState::Halted));
        assert_eq!(
            engine.registers.cpu.get(CPURegister::SP as usize).unwrap(),
            22
        );
        assert_eq!(
            engine.registers.cpu.get(CPURegister::GP as usize).unwrap(),
            55
        );
    }

    #[test]
    fn test_callback_disabled() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        // Callback
        let result = engine.callback();
        assert_eq!(result, Err(Error::CallbackNotEnabled));
    }
}
