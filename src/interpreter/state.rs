//! Embive Interpreter State

/// Embive Interpreter State
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    /// Interpreter running. Call [`super::Interpreter::run`] to continue running.
    Running,
    /// Interpreter was called (syscall). Optionally call [`super::Interpreter::syscall`] to handle the syscall and then [`super::Interpreter::run`] to continue running.
    Called,
    /// Interpreter waiting interrupt. Optionally call [`super::Interpreter::interrupt`] to trigger an interrupt and then [`super::Interpreter::run`] to continue running.
    Waiting,
    /// Interpreter halted. Call [`super::Interpreter::reset`] and then [`super::Interpreter::run`] to run again.
    Halted,
}

impl Default for State {
    fn default() -> Self {
        Self::Running
    }
}
