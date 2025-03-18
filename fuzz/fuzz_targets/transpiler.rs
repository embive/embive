#![no_main]
use libfuzzer_sys::fuzz_target;

use embive::transpiler::transpile_elf;

const MAX_SIZE: usize = 512;

fuzz_target!(|data: &[u8]| {
    // Code
    let mut code = [0; MAX_SIZE];

    // Convert RISC-V ELF to Embive binary
    let _ = transpile_elf(data, &mut code);
});
