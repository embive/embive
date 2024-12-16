//! Registers Module
mod control_status;
mod cpu;

#[doc(inline)]
pub use cpu::{CPURegister, CPURegisters};

#[doc(inline)]
pub use control_status::{CSOperation, CSRegisters};

/// Embive Registers
#[derive(Debug, Default, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub struct Registers {
    /// CPU Registers
    pub cpu: CPURegisters,
    /// Control and Status Registers
    pub control_status: CSRegisters,
}
