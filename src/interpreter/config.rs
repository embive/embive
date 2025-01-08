//! Embive Interpreter Config

/// Embive Interpreter Configuration Struct
#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// Instruction limit. Yield the interpreter when the limit is reached (0 = No limit).
    pub instruction_limit: u32,
}

impl Config {
    /// Set the instruction limit and return the configuration.
    /// Yield the interpreter when the limit is reached (0 = No limit).
    ///
    /// Arguments:
    /// - `instruction_limit`: Instruction limit.
    pub fn with_instruction_limit(mut self, instruction_limit: u32) -> Self {
        self.instruction_limit = instruction_limit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.instruction_limit, 0);
    }
}
