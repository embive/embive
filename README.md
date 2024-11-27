# Embive (Embedded RISC-V) [![Latest Version]][crates.io] [![docs]][docs.rs] [![msrv]][Rust 1.81]

[Latest Version]: https://img.shields.io/crates/v/embive.svg
[crates.io]: https://crates.io/crates/embive
[docs]: https://docs.rs/embive/badge.svg
[docs.rs]: https://docs.rs/embive
[msrv]: https://img.shields.io/crates/msrv/embive.svg?label=msrv&color=lightgray
[Rust 1.81]: https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html

Embive is a low-level sandboxing library focused on the embedding of untrusted code for constrained environments.  
As it interprets RISC-V bytecode, multiple languages are supported out of the box by Embive (Rust, C, C++, Zig, TinyGo, etc.).  
By default, it doesn’t require external crates, dynamic memory allocation or the standard library (`no_std` & `no_alloc`).

Embive is designed for any error during execution to be recoverable, allowing the host to handle it as needed.
As so, no panics should occur on release builds, despite the bytecode being executed.

Currently, it supports the `RV32IMZifencei` unprivileged instruction set.

## Templates
The following templates are available for programs that run inside Embive:
- [Rust template](https://github.com/embive/embive-rust-template)
- [C/C++ Template](https://github.com/embive/embive-c-template)

## Example
```rust
use embive::{engine::{Engine, Config, SYSCALL_ARGS}, memory::{Memory, SliceMemory}, register::Register};

/// A simple syscall example. Check [`engine::SyscallFn`] for more information.
fn syscall<M: Memory>(nr: i32, args: &[i32; SYSCALL_ARGS], memory: &mut M) -> Result<i32, i32> {
    println!("Syscall nr: {}, Args: {:?}", nr, args);
    match nr {
        1 => Ok(args[0] + args[1]), // Add two numbers (arg[0] + arg[1])
        2 => match memory.load(args[0] as u32) { // Load from RAM (arg[0])
            Ok(val) => Ok(i32::from_le_bytes(val)), // RISC-V is little endian
            Err(_) => Err(1),
        },
        _ => Err(2),
    }
}

fn main() {
    // "10 + 20" using syscalls (load from ram and add two numbers)
    let code = &[
        0x93, 0x08, 0x20, 0x00, // li   a7, 2      (Syscall nr = 2)
        0x13, 0x05, 0x10, 0x00, // li   a0, 1      (a0 = 1)
        0x13, 0x15, 0xf5, 0x01, // slli a0, a0, 31 (a0 << 31) (0x80000000)
        0x73, 0x00, 0x00, 0x00, // ecall           (Syscall, load from arg0)
        0x93, 0x08, 0x10, 0x00, // li   a7, 1      (Syscall nr = 1)
        0x13, 0x05, 0x40, 0x01, // li   a0,20      (a0 = 20)
        0x73, 0x00, 0x00, 0x00, // ecall           (Syscall, add two args)
        0x73, 0x00, 0x10, 0x00  // ebreak          (Halt)
    ];

    let mut ram = [0; 1024];
    // Store value 10 at RAM address 0 (0x80000000)
    ram[..4].copy_from_slice(&u32::to_le_bytes(10));

    // Create memory from code and RAM slices
    let mut memory = SliceMemory::new(code, &mut ram);

    // Create engine config
    let config = Config {
        syscall_fn: Some(syscall),
        ..Default::default()
    };

    // Create engine & run it
    let mut engine = Engine::new(&mut memory, config).unwrap();
    engine.run().unwrap();

    // Check the result (Ok(30))
    assert_eq!(engine.registers.get(Register::A0 as usize).unwrap(), 0);
    assert_eq!(engine.registers.get(Register::A1 as usize).unwrap(), 30);
}
```

## Roadmap
- [ ] Fully support `RV32IMACZifencei`
    - [x] RV32I Base Integer Instruction Set
    - [x] M Extension (Multiplication and Division Instructions)
    - [x] Zifencei
        - Implemented as a no-operation as it isn't applicable (Single HART, no cache, no memory-mapped devices, etc.).
    - [ ] A Extension (Atomic Instructions)
    - [ ] C Extension (Compressed Instructions)
- [x] System Calls
    - Function calls from interpreted to native code
- [x] Resource limiter
    - Yield the engine after a configurable amount of instructions are executed.
- [x] CI/CD
    - Incorporate more tests into the repository and create test automations for PRs
- [ ] Bytecode optimization (AOT and JIT)
    - Allow in-place JIT and AOT compilation to a format easier to parse.
        - Less bit-shifting, faster instruction matching, etc.
    - Should be kept as close as possible to native RISC-V bytecode.
- [ ] Callbacks
    - Function calls from native to interpreted code.
- [ ] Macros for converting native functions to system calls / callbacks
    - Use Rust type-system instead of only allowing `i32` arguments / results

## What about Floating Point?
Fully implementing the RISC-V F and/or D extensions would require using a soft-float library, as Rust doesn't 
support custom rounding modes nor does it expose the IEEE exception flags.

As the soft-float libraries available do not satisfy my requirements (must be portable, safe, no_std, and
support all rounding modes and exception flags), this feature will be halted until (if) an alternative is found.

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