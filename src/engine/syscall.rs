//! Embive Engine System Call
use core::num::NonZeroI32;

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
/// - `Result<i32, NonZeroI32>`: value (`a1`), error (`a0`).
pub type SyscallFn<M> =
    fn(nr: i32, args: &[i32; SYSCALL_ARGS], memory: &mut M) -> Result<i32, NonZeroI32>;
