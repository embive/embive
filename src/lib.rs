#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(all(feature = "interpreter", feature = "transpiler"), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/embive/embive/6da108bce7d0d01ac15ccb78786b68310c83289e/assets/embive_logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/embive/embive/6da108bce7d0d01ac15ccb78786b68310c83289e/assets/embive_logo.svg"
)]
//!
#![warn(missing_docs, rust_2018_idioms, future_incompatible, keyword_idents)]
#![deny(unsafe_code)]

#[cfg(all(feature = "alloc", feature = "transpiler"))]
extern crate alloc;

mod format;
pub mod instruction;
#[cfg(feature = "interpreter")]
pub mod interpreter;
#[cfg(feature = "transpiler")]
pub mod transpiler;

#[cfg(all(test, feature = "interpreter", feature = "transpiler"))]
mod tests {
    use core::num::NonZeroI32;
    use std::{
        fs::{read_dir, DirEntry},
        path::PathBuf,
    };

    use crate::{
        interpreter::{
            memory::{SliceMemory, RAM_OFFSET},
            Config, Error, Interpreter, State, SYSCALL_ARGS,
        },
        transpiler::transpile_elf,
    };

    const RAM_SIZE: usize = 32 * 1024;
    const RV32UI_TESTS: usize = 39;
    const RV32UM_TESTS: usize = 8;
    const RV32UA_TESTS: usize = 10;
    const RV32UC_TESTS: usize = 1;

    thread_local! {
        static SYSCALL_COUNTER: std::cell::RefCell<i32> = const { std::cell::RefCell::new(0) };
    }

    fn syscall(
        nr: i32,
        args: &[i32; SYSCALL_ARGS],
        _memory: &mut SliceMemory<'_>,
    ) -> Result<Result<i32, NonZeroI32>, Error> {
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
        Ok(Ok(0))
    }

    fn execute_bin_test(test: DirEntry) {
        let code = &[];

        println!("\nRunning: {}", test.file_name().to_string_lossy());

        // Load binary into RAM
        let mut ram = [0; RAM_SIZE];
        let test_elf = std::fs::read(test.path()).expect("Failed to read test file");
        transpile_elf(&test_elf, &mut ram).expect("Failed to transpile");

        let mut memory = SliceMemory::new(code, &mut ram);

        // Create interpreter
        let mut interpreter = Interpreter::new(
            &mut memory,
            Config {
                ..Default::default()
            },
        );

        // Set program counter to RAM (code start)
        interpreter.program_counter = RAM_OFFSET;

        // Get syscall counter prior to running
        let prev_syscall_counter = SYSCALL_COUNTER.with(|c| *c.borrow());

        // Run it
        loop {
            match interpreter.run().unwrap() {
                State::Running => {}
                State::Called => {
                    interpreter.syscall(&mut syscall).unwrap();
                }
                State::Waiting => {}
                State::Halted => break,
            }
        }

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
        dir.push("tests/riscv");
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

    #[test]
    fn rv32um_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests/riscv");
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

    #[test]
    fn rv32ua_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests/riscv");
        dir.push("rv32ua");

        let tests = read_dir(dir).expect("Failed to read directory");

        // Iterate over RV32UA tests
        let mut tested_files = 0;
        for test in tests {
            let test = test.expect("Failed to get test");
            execute_bin_test(test);
            tested_files += 1;
        }
        assert_eq!(tested_files, RV32UA_TESTS);
    }

    #[test]
    fn rv32uc_bin_tests() {
        // Get all tests
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("tests/riscv");
        dir.push("rv32uc");

        let tests = read_dir(dir).expect("Failed to read directory");

        // Iterate over RV32UC tests
        let mut tested_files = 0;
        for test in tests {
            let test = test.expect("Failed to get test");
            execute_bin_test(test);
            tested_files += 1;
        }
        assert_eq!(tested_files, RV32UC_TESTS);
    }
}
