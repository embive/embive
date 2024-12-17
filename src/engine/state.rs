//! Embive Engine State

/// Embive Engine State
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EngineState {
    /// Engine running (yielded for some reason, ex: instruction limit reached). Call [`super::Engine::run`] to continue.
    Running,
    /// Engine waiting callback/event (WFI instruction). Call [`super::Engine::callback`] and then [`super::Engine::run`] to continue.
    Waiting,
    /// Engine halted (ebreak instruction). Call [`super::Engine::reset`] and then [`super::Engine::run`] to run again.
    Halted,
}
