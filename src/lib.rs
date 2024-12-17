#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/embive/embive/6da108bce7d0d01ac15ccb78786b68310c83289e/assets/embive_logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/embive/embive/6da108bce7d0d01ac15ccb78786b68310c83289e/assets/embive_logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms, future_incompatible, keyword_idents)]
#![deny(unsafe_code)]

pub mod engine;
mod error;
mod instruction;
pub mod memory;
pub mod registers;

#[doc(inline)]
pub use error::Error;

#[cfg(test)]
mod tests {
    use std::{
        fs::{read_dir, DirEntry},
        path::PathBuf,
    };

    use crate::{
        engine::{Config, Engine, SYSCALL_ARGS},
        memory::{SliceMemory, RAM_OFFSET},
    };

    const RAM_SIZE: usize = 16 * 1024;
    const RV32UI_TESTS: usize = 40;
    #[cfg(feature = "m_extension")]
    const RV32UM_TESTS: usize = 8;
    #[cfg(feature = "a_extension")]
    const RV32UA_TESTS: usize = 10;

    thread_local! {
        static SYSCALL_COUNTER: std::cell::RefCell<i32> = std::cell::RefCell::new(0);
    }

    fn syscall(
        nr: i32,
        args: &[i32; SYSCALL_ARGS],
        _memory: &mut SliceMemory<'_>,
    ) -> Result<i32, i32> {
        if nr == 93 {
            if args[0] == 0 {
                println!("Test was successful");
            } else {
                panic!("Failed test number: {}", args[0] >> 1);
            }
        } else {
            panic!("Unknown syscall: {}", nr);
        }

        SYSCALL_COUNTER.with(|c| *c.borrow_mut() += 1);
        Ok(0)
    }

    fn execute_bin_test(test: DirEntry) {
        let code = &[];

        println!("\nRunning: {}", test.file_name().to_string_lossy());

        // Load binary into RAM
        let mut ram = [0; RAM_SIZE];
        let test_bytes = std::fs::read(test.path()).expect("Failed to read test file");
        ram[..test_bytes.len()].copy_from_slice(&test_bytes);

        let mut memory = SliceMemory::new(code, &mut ram);

        // Create engine
        let mut engine = Engine::new(
            &mut memory,
            Config {
                syscall_fn: Some(syscall),
                ..Default::default()
            },
        )
        .unwrap();

        // Set program counter to RAM (code start)
        engine.program_counter = RAM_OFFSET;

        // Get syscall counter prior to running
        let prev_syscall_counter = SYSCALL_COUNTER.with(|c| *c.borrow());

        // Run it
        engine.run().unwrap();

        // Get syscall counter after running
        let new_syscall_counter = SYSCALL_COUNTER.with(|c| *c.borrow());

        // Check if syscall was incremented
        if new_syscall_counter <= prev_syscall_counter {
            panic!("No syscall was made");
        }
    }

    #[test]
    fn rv32ui_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests");
        dir.push("rv32ui");

        let tests = read_dir(dir).expect("Failed to read directory");

        // Iterate over RV32UI tests
        let mut tested_files = 0;
        for test in tests {
            let test = test.expect("Failed to get test");
            execute_bin_test(test);
            tested_files += 1;
        }
        assert_eq!(tested_files, RV32UI_TESTS);
    }

    #[cfg(feature = "m_extension")]
    #[test]
    fn rv32um_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests");
        dir.push("rv32um");

        let tests = read_dir(dir).expect("Failed to read directory");

        // Iterate over RV32UM tests
        let mut tested_files = 0;
        for test in tests {
            let test = test.expect("Failed to get test");
            execute_bin_test(test);
            tested_files += 1;
        }
        assert_eq!(tested_files, RV32UM_TESTS);
    }

    #[cfg(feature = "a_extension")]
    #[test]
    fn rv32ua_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests");
        dir.push("rv32ua");

        let tests = read_dir(dir).expect("Failed to read directory");

        // Iterate over RV32UM tests
        let mut tested_files = 0;
        for test in tests {
            let test = test.expect("Failed to get test");
            execute_bin_test(test);
            tested_files += 1;
        }
        assert_eq!(tested_files, RV32UA_TESTS);
    }
}
