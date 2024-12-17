//! Embive Engine Config

use crate::memory::Memory;

use super::SyscallFn;

/// Embive Engine Configuration Struct
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub struct Config<M: Memory> {
    /// System call function (Called by `ecall` instruction).
    pub syscall_fn: Option<SyscallFn<M>>,
    /// Instruction limit. Yield when the limit is reached (0 = No limit).
    pub instruction_limit: u32,
}

impl<M: Memory> Config<M> {
    /// Set the system call function and return the configuration.
    ///
    /// Arguments:
    /// - `syscall_fn`: Optional system call function.
    pub fn with_syscall_fn(mut self, syscall_fn: Option<SyscallFn<M>>) -> Self {
        self.syscall_fn = syscall_fn;
        self
    }

    /// Set the instruction limit and return the configuration.
    ///
    /// Arguments:
    /// - `instruction_limit`: Instruction limit (0 = No limit).
    pub fn with_instruction_limit(mut self, instruction_limit: u32) -> Self {
        self.instruction_limit = instruction_limit;
        self
    }
}

impl<M: Memory> Default for Config<M> {
    fn default() -> Self {
        Config {
            syscall_fn: None,
            instruction_limit: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::SliceMemory;

    #[test]
    fn test_default_config() {
        let config = Config::<SliceMemory<'_>>::default();
        assert_eq!(config.syscall_fn, None);
        assert_eq!(config.instruction_limit, 0);
    }
}
