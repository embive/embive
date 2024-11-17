# Embive (Embedded RISC-V)
Embive is a low-level sandboxing library focused on the embedding of untrusted code for constrained environments.  
As it interprets RISC-V bytecode, multiple languages are supported out of the box by Embive (Rust, C, C++, Zig, TinyGo, etc.).  
By default, Embive doesn’t require any external crate, dynamic memory allocation or the standard library (`no_std` and `no_alloc`).

Currently, it supports the `RV32I[M]` unprivileged instruction set (M extension enabled by default).

## Example
```rust,ignore
use embive::engine::{memory::Memory, register::Register, Engine, SYSCALL_ARGS};

// A simple syscall example. Check [`engine::SyscallFn`] for more information.
fn syscall(nr: i32, args: [i32; SYSCALL_ARGS], memory: &mut Memory) -> (i32, i32) {
    println!("{}: {:?}", nr, args);
    match nr {
        1 => (args[0] + args[1], 0), // Add two numbers (arg[0] + arg[1])
        2 => match memory.load(args[0] as u32) { // Load from RAM (arg[0])
            Ok(val) => (i32::from_le_bytes(val), 0),
            Err(_) => (0, 1),
        },
        _ => (0, 2),
    }
}

fn main() {
    // "10 + 20" using syscalls (load from ram and add two numbers)
    let code = &[
        0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr)
        0x13, 0x05, 0x10, 0x00, // li   a0, 1      (arg0, set first bit)
        0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (arg0, shift-left 31 bits)
        0x73, 0x00, 0x00, 0x00, // ecall           (Syscall, load from arg0)
        0x93, 0x08, 0x10, 0x00, // li   a7, 1      (Syscall nr)
        0x93, 0x05, 0x40, 0x01, // li   a1,20      (arg1, 20)
        0x73, 0x00, 0x00, 0x00, // ecall           (Syscall, add two args)
        0x73, 0x00, 0x10, 0x00  // ebreak          (Halt, exit VM)
    ];
    let mut ram = [0; 1024];
    ram[..4].copy_from_slice(&u32::to_le_bytes(10));

    // Create engine
    let mut engine = Engine::new(code, &mut ram, Some(syscall)).unwrap();

    // Run it
    engine.run().unwrap();

    // Check the result
   assert_eq!(engine.registers().get(Register::A0 as usize).unwrap(), 30);
   assert_eq!(engine.registers().get(Register::A1 as usize).unwrap(), 0);
}
```

## Roadmap
- [ ] Fully support `RV32G` (RV32IMAFDZicsr_Zifencei)
    - [x] RV32I Base Integer Instruction Set
    - [x] M Extension (Multiplication and Division Instructions)
    - [x] Zifencei
        - Implemented as a no-operation as it isn't applicable (Single HART, no cache, no memory-mapped devices, etc.).
    - [ ] Zicsr
        - At least the unprivileged CSRs
    - [ ] F Extension (Single-Precision Floating-Point Instructions)
    - [ ] D Extension (Double-Precision Floating-Point Instructions)
    - [ ] A Extension (Atomic Instructions)
- [x] System Calls
    - Function calls from interpreted to native code
- [ ] Resource limiter
    - Yield the engine after a configurable amount of instructions are executed.
- [ ] CI/CD
    - Incorporate more tests into the repository and create test automations for PRs
- [ ] Bytecode optimization (AOT and JIT)
    - Allow in-place JIT and AOT compilation to a format easier to parse.
        - Less bit-shifting, faster instruction matching, etc.
    - Should be kept as close as possible to native RISC-V bytecode.
- [ ] Callbacks
    - Function calls from native to interpreted code.
- [ ] Macros for converting native functions to system calls / callbacks
    - Use Rust type-system instead of only allowing `i32` arguments / results
- [ ] Support C Extension (Compressed Instructions)
    - This is a maybe, but good to keep in mind while developing other features (especially AOT/JIT).


## Minimum supported Rust version (MSRV)
Embive is guaranteed to compile on stable Rust 1.81 and up.

## License

Embive is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.