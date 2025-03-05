//! Embive Debugger
mod gdb;

use core::{marker::PhantomData, num::NonZeroI32};

use gdbstub::{
    common::Signal,
    conn::ConnectionExt,
    stub::{
        run_blocking::{self, BlockingEventLoop},
        SingleThreadStopReason,
    },
};

use super::{memory::Memory, Interpreter, State, SYSCALL_ARGS};

/// Debugger Execution Mode
#[derive(Debug, PartialEq)]
enum ExecMode {
    Step,
    Run,
}

/// A debugger based on gdbstub for the embive interpreter.
///
/// Generics:
/// - `'a`: Lifetime of the interpreter
/// - `M`: Memory type
/// - `C`: Connection type
/// - `F`: Syscall function type
/// - `N`: Maximum number of breakpoints
#[derive(Debug)]
pub struct Debugger<
    'a,
    M: Memory,
    C: ConnectionExt,
    F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<i32, NonZeroI32>,
    const N: usize = 4,
> {
    interpreter: Interpreter<'a, M>,
    breakpoints: [Option<u32>; N],
    exec_mode: ExecMode,
    syscall_fn: F,
    _conn: PhantomData<C>,
}

impl<
        'a,
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<i32, NonZeroI32>,
        const N: usize,
    > From<Debugger<'a, M, C, F, N>> for Interpreter<'a, M>
{
    fn from(debugger: Debugger<'a, M, C, F, N>) -> Self {
        debugger.interpreter
    }
}

impl<
        'a,
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<i32, NonZeroI32>,
        const N: usize,
    > Debugger<'a, M, C, F, N>
{
    /// Create a new debugger for the given memory and syscall function.
    pub fn new(memory: &'a mut M, syscall_fn: F) -> Self {
        Self {
            interpreter: Interpreter::new(memory, super::Config::default()),
            breakpoints: [None; N],
            exec_mode: ExecMode::Run,
            syscall_fn,
            _conn: PhantomData,
        }
    }
}

impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<i32, NonZeroI32>,
        const N: usize,
    > BlockingEventLoop for Debugger<'_, M, C, F, N>
{
    type Target = Self;
    type Connection = C;
    type StopReason = SingleThreadStopReason<u32>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        gdbstub::stub::run_blocking::Event<Self::StopReason>,
        gdbstub::stub::run_blocking::WaitForStopReasonError<
            <Self::Target as gdbstub::target::Target>::Error,
            <Self::Connection as gdbstub::conn::Connection>::Error,
        >,
    > {
        let mut cycles = 0;
        loop {
            // Run a single instruction.
            match target
                .interpreter
                .step()
                .map_err(run_blocking::WaitForStopReasonError::Target)?
            {
                State::Running => (),
                State::Halted => {
                    return Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::Terminated(Signal::SIGSTOP),
                    ))
                }
                State::Called => target.interpreter.syscall(&mut target.syscall_fn),
                State::Waiting => target
                    .interpreter
                    .interrupt()
                    .map_err(run_blocking::WaitForStopReasonError::Target)?,
            }

            // Check for breakpoints at the current program counter.
            if target
                .breakpoints
                .contains(&Some(target.interpreter.program_counter))
            {
                return Ok(run_blocking::Event::TargetStopped(
                    SingleThreadStopReason::SwBreak(()),
                ));
            }

            // Step mode stops after one instruction.
            if target.exec_mode == ExecMode::Step {
                return Ok(run_blocking::Event::TargetStopped(
                    SingleThreadStopReason::DoneStep,
                ));
            }

            // Every 1024 instructions, check for incoming data.
            if cycles % 1024 == 0 && conn.peek().map(|b| b.is_some()).unwrap_or(true) {
                let byte = conn
                    .read()
                    .map_err(run_blocking::WaitForStopReasonError::Connection)?;

                return Ok(run_blocking::Event::IncomingData(byte));
            }
            cycles += 1;
        }
    }

    fn on_interrupt(
        _target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as gdbstub::target::Target>::Error> {
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}
