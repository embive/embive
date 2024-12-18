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

/// Embive Custom Interrupt Code
pub const EMBIVE_INTERRUPT_CODE: u32 = 16;

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

    /// Run the engine, executing the interpreted code.
    ///
    /// Returns:
    /// - `Ok(EngineState)`: Success, current engine state (check [`EngineState`]).
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
    /// - `Ok(EngineState)`: Success, current engine state (check [`EngineState`]).
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

    /// Execute an interrupt as configured by the interpreted code.
    /// This call does not run any interpreted code, [`Engine::run`] should be called after.
    /// Interrupt must be configured/enabled by the interpreted code for this function to succeed.
    ///
    /// Interrupt traps are enabled by setting CSRs `mstatus.MIE` and `mie` bit [`EMBIVE_INTERRUPT_CODE`], as well as
    /// configuring `mtvec` with a valid address. If done correctly, the engine will set the interrupt pending bit
    /// (`mip` bit [`EMBIVE_INTERRUPT_CODE`]) and jump to the address in `stvec` when an interrupt is triggered.
    ///
    /// The interrupt pending (`mip`) bit [`EMBIVE_INTERRUPT_CODE`] can be cleared by manually writing 0 to it.
    ///
    /// Returns:
    /// - `Ok(())`: Success, interrupt executed.
    /// - `Err(Error)`: Interrupt not enabled by interpreted code.
    pub fn interrupt(&mut self) -> Result<(), Error> {
        // Check if interrupt is enabled
        if !self.registers.control_status.interrupt_enabled() {
            // Interrupt is not enabled
            return Err(Error::InterruptNotEnabled);
        }

        // Set interrupt
        self.registers.control_status.set_interrupt();

        // Trap to the interrupt handler
        self.registers
            .control_status
            .trap_entry(&mut self.program_counter);

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
                    self.registers.cpu.inner[CPURegister::A0 as usize] = error.into();

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
    use core::num::NonZeroI32;

    use crate::memory::SliceMemory;

    use super::*;

    fn syscall<M: Memory>(
        nr: i32,
        args: &[i32; SYSCALL_ARGS],
        _memory: &mut M,
    ) -> Result<i32, NonZeroI32> {
        // Match the syscall number
        match nr {
            0 => Ok(0),
            1 => {
                // Check all 7 arguments
                if args[0] == 1
                    && args[1] == 2
                    && args[2] == 3
                    && args[3] == 4
                    && args[4] == -5
                    && args[5] == -6
                    && args[6] == -7
                {
                    Ok(-1)
                } else {
                    Err((-1i32).try_into().unwrap())
                }
            }
            _ => Err(1.try_into().unwrap()), // Not implemented
        }
    }

    #[test]
    fn test_syscall() {
        let code = &[
            0x93, 0x08, 0x00, 0x00, // li   a7, 0
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(code, &mut []);

        // Create engine config
        let config = Config::default().with_syscall_fn(Some(syscall));

        // Create engine & run it
        let mut engine = Engine::new(&mut memory, config).unwrap();
        engine.run().unwrap();

        // Check the result (Ok(0))
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A0 as usize).unwrap(),
            0
        );
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A1 as usize).unwrap(),
            0
        );
    }

    #[test]
    fn test_syscall_error() {
        let code = &[
            0x93, 0x08, 0x20, 0x00, // li   a7, 2
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(code, &mut []);

        // Create engine config
        let config = Config::default().with_syscall_fn(Some(syscall));

        // Create engine & run it
        let mut engine = Engine::new(&mut memory, config).unwrap();
        engine.run().unwrap();

        // Check the result (Err(1))
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A0 as usize).unwrap(),
            1
        );
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A1 as usize).unwrap(),
            0
        );
    }

    #[test]
    fn test_syscall_args() {
        let code = &[
            0x93, 0x08, 0x10, 0x00, // li   a7, 1
            0x13, 0x05, 0x10, 0x00, // li   a0, 1
            0x93, 0x05, 0x20, 0x00, // li   a1, 2
            0x13, 0x06, 0x30, 0x00, // li   a2, 3
            0x93, 0x06, 0x40, 0x00, // li   a3, 4
            0x13, 0x07, 0xb0, 0xff, // li   a4, -5
            0x93, 0x07, 0xa0, 0xff, // li   a5, -6
            0x13, 0x08, 0x90, 0xff, // li   a6, -7
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(code, &mut []);

        // Create engine config
        let config = Config::default().with_syscall_fn(Some(syscall));

        // Create engine & run it
        let mut engine = Engine::new(&mut memory, config).unwrap();
        engine.run().unwrap();

        // Check the result (Ok(-1))
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A0 as usize).unwrap(),
            0
        );
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A1 as usize).unwrap(),
            -1
        );
    }

    #[test]
    fn test_syscall_args_error() {
        let code = &[
            0x93, 0x08, 0x10, 0x00, // li   a7, 1
            0x13, 0x05, 0xf0, 0xff, // li   a0, -1
            0x93, 0x05, 0xe0, 0xff, // li   a1, -2
            0x13, 0x06, 0xd0, 0xff, // li   a2, -3
            0x93, 0x06, 0xc0, 0xff, // li   a3, -4
            0x13, 0x07, 0x50, 0x00, // li   a4, 5
            0x93, 0x07, 0x60, 0x00, // li   a5, 6
            0x13, 0x08, 0x70, 0x00, // li   a6, 7
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(code, &mut []);

        // Create engine config
        let config = Config::default().with_syscall_fn(Some(syscall));

        // Create engine & run it
        let mut engine = Engine::new(&mut memory, config).unwrap();
        engine.run().unwrap();

        // Check the result (Err(-1))
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A0 as usize).unwrap(),
            -1
        );
        assert_eq!(
            engine.registers.cpu.get(CPURegister::A1 as usize).unwrap(),
            0
        );
    }

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
    fn test_interrupt() {
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
        let mut engine = Engine::new(&mut memory, Config::default()).unwrap();

        // Run the engine
        let result = engine.run();
        assert_eq!(result, Ok(EngineState::Waiting));
        assert_eq!(
            engine.registers.cpu.get(CPURegister::SP as usize).unwrap(),
            55
        );

        // interrupt
        let result = engine.interrupt();
        assert_eq!(result, Ok(()));
        assert_eq!(engine.program_counter, 40);
        assert!(
            engine
                .registers
                .control_status
                .operation(None, 0x344) // MIP
                .unwrap()
                & (1 << EMBIVE_INTERRUPT_CODE)
                != 0
        );

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
    fn test_interrupt_disabled() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut engine = Engine::new(&mut memory, Default::default()).unwrap();

        // interrupt
        let result = engine.interrupt();
        assert_eq!(result, Err(Error::InterruptNotEnabled));
    }
}
