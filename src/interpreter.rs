//! Interpreter Module
//!
//! This module contains the Embive interpreter, which is responsible for executing the interpreted code.
//! It uses the Embive instruction set and provides a simple interface for running and debugging the code.
#[cfg(feature = "debugger")]
mod debugger;
mod decode_execute;
mod error;
pub mod memory;
pub mod registers;
mod state;

use core::num::NonZeroI32;

use decode_execute::decode_execute;
use memory::Memory;
use registers::{CPURegister, Registers};

#[doc(inline)]
pub use error::Error;
#[doc(inline)]
pub use state::State;

#[cfg(feature = "debugger")]
#[doc(inline)]
pub use debugger::Debugger;

use crate::instruction::embive::Instruction;

/// Embive Custom Interrupt Code
pub const EMBIVE_INTERRUPT_CODE: u32 = 16;

/// Number of syscall arguments
pub const SYSCALL_ARGS: usize = 7;

/// Embive Interpreter Struct
#[derive(Debug)]
#[non_exhaustive]
pub struct Interpreter<'a, M: Memory> {
    /// Program Counter.
    pub program_counter: u32,
    /// CPU Registers.
    pub registers: Registers,
    /// System Memory (code + RAM).
    pub memory: &'a mut M,
    /// Instruction limit (0 means no limit).
    pub instruction_limit: u32,
    /// Memory reservation for atomic operations (addr, value).
    pub(crate) memory_reservation: Option<(u32, i32)>,
}

impl<'a, M: Memory> Interpreter<'a, M> {
    /// Create a new interpreter.
    ///
    /// Arguments:
    /// - `memory`: System memory (code + RAM).
    /// - `instruction_limit`: Execution will yield when the instruction limit is reached (0 means no limit).
    pub fn new(memory: &'a mut M, instruction_limit: u32) -> Self {
        // Create the interpreter
        Interpreter {
            program_counter: 0,
            registers: Default::default(),
            memory,
            instruction_limit,
            memory_reservation: None,
        }
    }

    /// Reset the interpreter:
    /// - Program counter is reset to 0.
    /// - CPU Registers are reset to 0.
    /// - Memory reservation is cleared.
    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.registers = Default::default();
        self.memory_reservation = None;
    }

    /// Run the interpreter, executing the code.
    ///
    /// Returns:
    /// - `Ok(State)`: Success, current state (check [`State`]).
    /// - `Err(Error)`: Failed to run.
    pub fn run(&mut self) -> Result<State, Error> {
        // Check if there is an instruction limit
        if self.instruction_limit > 0 {
            // Run the interpreter with an instruction limit
            for _ in 0..self.instruction_limit {
                // Step through the program
                let state = self.step()?;

                if state != State::Running {
                    // Stop running
                    return Ok(state);
                }
            }

            // Yield after the instruction limit (still running)
            return Ok(State::Running);
        }

        // No instruction limit
        loop {
            // Step through the program
            let state = self.step()?;

            if state != State::Running {
                // Stop running
                return Ok(state);
            }
        }
    }

    /// Step through a single instruction from the current program counter.
    ///
    /// Returns:
    /// - `Ok(State)`: Success, current state (check [`State`]).
    /// - `Err(Error)`: Failed to execute.
    #[inline(always)]
    pub fn step(&mut self) -> Result<State, Error> {
        // Fetch next instruction
        let data = u32::from(self.fetch()?);

        // Decode and execute the instruction
        let ret = decode_execute(self, data)?;

        Ok(ret)
    }

    /// Fetch the next instruction from the program counter.
    ///
    /// Returns:
    /// - `Ok(Instruction)`: The instruction that was fetched.
    /// - `Err(Error)`: The program counter is out of bounds.
    #[inline(always)]
    pub fn fetch(&mut self) -> Result<Instruction, Error> {
        let data = self.memory.load(self.program_counter, 4)?;
        // Unwrap is safe because the slice is guaranteed to have 4 elements.
        Ok(u32::from_le_bytes(data.try_into().unwrap()).into())
    }

    /// Execute an interrupt as configured by the interpreted code.
    /// This call does not run any interpreted code, [`Interpreter::run`] should be called after.
    /// Interrupt must be configured/enabled by the interpreted code for this function to succeed.
    ///
    /// Interrupt traps are enabled by setting CSRs `mstatus.MIE` and `mie` bit [`EMBIVE_INTERRUPT_CODE`], as well as
    /// configuring `mtvec` with a valid address. If done correctly, the interpreter will set the interrupt pending bit
    /// (`mip` bit [`EMBIVE_INTERRUPT_CODE`]) and jump to the address in `stvec` when an interrupt is triggered.
    ///
    /// The interrupt pending (`mip`) bit [`EMBIVE_INTERRUPT_CODE`] can be cleared by manually writing 0 to it.
    ///
    /// Arguments:
    /// - `value`: Value to be passed to the interrupt handler (through `mtval` CSR).
    ///
    /// Returns:
    /// - `Ok(())`: Success, interrupt executed.
    /// - `Err(Error)`: Interrupt not enabled by interpreted code.
    pub fn interrupt(&mut self, value: i32) -> Result<(), Error> {
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
            .trap_entry(&mut self.program_counter, value);

        Ok(())
    }

    /// Get the syscall arguments.
    #[inline(always)]
    fn syscall_arguments(&mut self) -> (i32, &[i32; SYSCALL_ARGS], &mut M) {
        // Syscall Number
        let nr = self.registers.cpu.inner[CPURegister::A7 as usize];

        // Syscall Arguments
        let args = self.registers.cpu.inner[CPURegister::A0 as usize..]
            .first_chunk()
            // Unwrap is safe because the slice is guaranteed to have more than SYSCALL_ARGS elements.
            .unwrap();

        (nr, args, self.memory)
    }

    /// Set the syscall result.
    #[inline(always)]
    fn syscall_result(&mut self, result: Result<i32, NonZeroI32>) {
        match result {
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
    }

    /// Handle a system call.
    ///
    /// System calls are triggered by the `ecall` instruction.
    /// The following registers are used:
    /// - `a7`: Syscall number.
    /// - `a0` to `a6`: Arguments.
    /// - `a0`: Return error code.
    /// - `a1`: Return value.
    ///
    /// Arguments:
    /// - `function`: System call function (FnMut closure):
    ///     - Arguments:
    ///         - `i32`: Syscall number (`a7`).
    ///         - `[i32; SYSCALL_ARGS]`: Arguments (`a0` to `a6`).
    ///         - `Memory`: System Memory (code + RAM).
    ///
    ///     - Returns:
    ///         - `Result<Result<i32, NonZeroI32>, E>`:
    ///             - Outer `Result`: Ok(()) if the syscall was successful, Err(E) if an internal error occurred. Errors are returned to the calling code.
    ///             - Inner `Result`: Mapped to the value (`a1`) and error (`a0`) returned to the interpreted code.
    pub fn syscall<F, E>(&mut self, function: &mut F) -> Result<(), E>
    where
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, E>,
    {
        // Get syscall arguments
        let (nr, args, memory) = self.syscall_arguments();

        // Call the syscall function
        let result = function(nr, args, memory)?;

        // Set the syscall result
        self.syscall_result(result);

        Ok(())
    }

    /// Handle a system call asynchronously.
    ///
    /// System calls are triggered by the `ecall` instruction.
    /// The following registers are used:
    /// - `a7`: Syscall number.
    /// - `a0` to `a6`: Arguments.
    /// - `a0`: Return error code.
    /// - `a1`: Return value.
    ///
    /// Arguments:
    /// - `function`: System call function (AsyncFnMut closure):
    ///     - Arguments:
    ///         - `i32`: Syscall number (`a7`).
    ///         - `[i32; SYSCALL_ARGS]`: Arguments (`a0` to `a6`).
    ///         - `Memory`: System Memory (code + RAM).
    ///
    ///     - Returns:
    ///         - `Result<Result<i32, NonZeroI32>, E>`:
    ///             - Outer `Result`: Ok(()) if the syscall was successful, Err(E) if an internal error occurred. Errors are returned to the calling code.
    ///             - Inner `Result`: Mapped to the value (`a1`) and error (`a0`) returned to the interpreted code.
    #[cfg(feature = "async")]
    pub async fn syscall_async<F, E>(&mut self, function: &mut F) -> Result<(), E>
    where
        F: AsyncFnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, E>,
    {
        // Get syscall arguments
        let (nr, args, memory) = self.syscall_arguments();

        // Call the syscall function
        let result = function(nr, args, memory).await?;

        // Set the syscall result
        self.syscall_result(result);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "transpiler")]
    use core::num::NonZeroI32;
    use memory::SliceMemory;

    #[cfg(feature = "transpiler")]
    use crate::transpiler::transpile_raw;

    use super::*;

    #[cfg(feature = "transpiler")]
    fn syscall(
        nr: i32,
        args: &[i32; SYSCALL_ARGS],
        _memory: &mut SliceMemory<'_>,
    ) -> Result<Result<i32, NonZeroI32>, Error> {
        // Match the syscall number
        Ok(match nr {
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
        })
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_syscall() {
        let mut code = [
            0x93, 0x08, 0x00, 0x00, // li   a7, 0
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];
        transpile_raw(&mut code).unwrap();

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(&code, &mut []);

        // Create interpreter & run it
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let state = interpreter.run().unwrap();

        // Host Called (syscall)
        assert_eq!(state, State::Called);
        interpreter.syscall(&mut syscall).unwrap();

        // Check the result (Ok(0))
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A0 as u8)
                .unwrap(),
            0
        );
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A1 as u8)
                .unwrap(),
            0
        );
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_syscall_error() {
        let mut code = [
            0x93, 0x08, 0x20, 0x00, // li   a7, 2
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];
        transpile_raw(&mut code).unwrap();

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(&code, &mut []);

        // Create interpreter & run it
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let state = interpreter.run().unwrap();

        // Host Called (syscall)
        assert_eq!(state, State::Called);
        interpreter.syscall(&mut syscall).unwrap();

        // Check the result (Err(1))
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A0 as u8)
                .unwrap(),
            1
        );
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A1 as u8)
                .unwrap(),
            0
        );
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_syscall_args() {
        let mut code = [
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
        transpile_raw(&mut code).unwrap();

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(&code, &mut []);

        // Create interpreter & run it
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let state = interpreter.run().unwrap();

        // Host Called (syscall)
        assert_eq!(state, State::Called);
        interpreter.syscall(&mut syscall).unwrap();

        // Check the result (Ok(-1))
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A0 as u8)
                .unwrap(),
            0
        );
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A1 as u8)
                .unwrap(),
            -1
        );
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_syscall_args_error() {
        let mut code = [
            0x93, 0x08, 0x10, 0x00, // li   a7, 1
            0x13, 0x05, 0xf0, 0xff, // li   a0, -1
            0x93, 0x05, 0xe0, 0xff, // li   a1, -2
            0x13, 0x06, 0xd0, 0xff, // li   a2, -3
            0x93, 0x06, 0xc0, 0xff, // li   a3, -4
            0x13, 0x07, 0x50, 0x00, // li   a4, 5
            0x93, 0x07, 0x60, 0x00, // li   a5, 6
            0x13, 0x08, 0x70, 0x00, // li   a6, 7
            0x0f, 0x10, 0x00, 0x00, // Fence.i (nop)
            0x73, 0x00, 0x00, 0x00, // ecall
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];
        transpile_raw(&mut code).unwrap();

        // Create memory from code and RAM slices
        let mut memory = SliceMemory::new(&code, &mut []);

        // Create interpreter & run it
        let mut interpreter = Interpreter::new(&mut memory, 0);
        let state = interpreter.run().unwrap();

        // Host Called (syscall)
        assert_eq!(state, State::Called);
        interpreter.syscall(&mut syscall).unwrap();

        // Check the result (Err(-1))
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A0 as u8)
                .unwrap(),
            -1
        );
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::A1 as u8)
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_reset() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);
        interpreter.reset();

        assert_eq!(interpreter.program_counter, 0);
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_instruction_limit() {
        let mut code = [
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];
        transpile_raw(&mut code).unwrap();

        let mut memory = SliceMemory::new(&code, &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 2);

        // Run the interpreter
        let result = interpreter.run();
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.program_counter, 4 * 2);

        // Run the interpreter again
        let result = interpreter.run();
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(interpreter.program_counter, 4 * 4);
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_instruction_limit_zero() {
        let mut code = [
            0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
            0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
            0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
            0x73, 0x00, 0x10, 0x00, // ebreak          (Halt)
        ];
        transpile_raw(&mut code).unwrap();

        let mut memory = SliceMemory::new(&code, &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);

        // Run the interpreter
        let result = interpreter.run();
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(interpreter.program_counter, 4 * 4);
    }

    #[cfg(feature = "transpiler")]
    #[test]
    fn test_interrupt() {
        let mut code = [
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
        transpile_raw(&mut code).unwrap();

        let mut memory = SliceMemory::new(&code, &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);

        // Run the interpreter
        let result = interpreter.run();
        assert_eq!(result, Ok(State::Waiting));
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::SP as u8)
                .unwrap(),
            55
        );

        // interrupt
        let result = interpreter.interrupt(1024);
        assert_eq!(result, Ok(()));
        assert_eq!(interpreter.program_counter, 40);
        assert!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x344) // MIP
                .unwrap()
                & (1 << EMBIVE_INTERRUPT_CODE)
                != 0
        );
        assert_eq!(
            interpreter
                .registers
                .control_status
                .operation(None, 0x343) // MTVAL
                .unwrap(),
            1024
        );

        // Run the interpreter again
        let result = interpreter.run();
        assert_eq!(result, Ok(State::Halted));
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::SP as u8)
                .unwrap(),
            22
        );
        assert_eq!(
            interpreter
                .registers
                .cpu
                .get(CPURegister::GP as u8)
                .unwrap(),
            55
        );
    }

    #[test]
    fn test_interrupt_disabled() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, 0);

        // interrupt
        let result = interpreter.interrupt(0);
        assert_eq!(result, Err(Error::InterruptNotEnabled));
    }
}
