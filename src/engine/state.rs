//! Embive Engine State

/// Embive Engine State
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EngineState {
    /// Engine running. Call [`super::Engine::run`] to continue running.
    Running,
    /// Engine waiting interrupt. Call [`super::Engine::run`] to continue running. (Optionally call [`super::Engine::interrupt`] prior to trigger an interrupt).
    Waiting,
    /// Engine halted. Call [`super::Engine::reset`] and then [`super::Engine::run`] to run again.
    Halted,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::Running
    }
}
