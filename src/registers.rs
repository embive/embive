//! Registers Module
mod cpu;

#[doc(inline)]
pub use cpu::{CPURegister, CPURegisters};

/// CPU Registers
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Registers {
    pub cpu: CPURegisters,
}
