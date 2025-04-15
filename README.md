# Embive (Embedded RISC-V) [![Latest Version]][crates.io] [![docs]][docs.rs] [![msrv]][Rust 1.81]

[Latest Version]: https://img.shields.io/crates/v/embive.svg
[crates.io]: https://crates.io/crates/embive
[docs]: https://docs.rs/embive/badge.svg
[docs.rs]: https://docs.rs/embive
[msrv]: https://img.shields.io/crates/msrv/embive.svg?label=msrv&color=lightgray
[Rust 1.81]: https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html

A lightweight, recoverable sandbox for executing untrusted RISC-V code in constrained environments (ex.: microcontrollers).

🛡️ **Secure**: No unsafe code, no panics on release builds.  
📦 **Embeddable**: No standard library required.  
⚡ **Deterministic**: No heap allocation required.  

This library was inspired by [WebAssembly](https://webassembly.org/) and [Q3VM](https://www.icculus.org/~phaethon/q3mc/q3vm_specs.html):
Allow compiled code to be dinamically loaded by a host application, while also restricting memory access and resource usage.  

## How It Works

Embive supports the RISC-V `RV32IMAC` instruction-set, making it compatible with many available languages and toolchains.

For better performance at the target device, Embive uses a two-stage execution model:
1. Transpilation ([more info. here](TRANSPILER.md))  
   Converts RISC-V ELF file to an optimized bytecode binary:
    - Reorder immediates
    - Expand compressed registers wherever possible
    - Simplify instruction matching
2. Interpretation  
   Executes the bytecode using a register-based virtual machine with:
    - Memory isolation
    - Syscall and interruption support
    - Instruction limiting

The transpiled bytecode is stable and can be executed by any device running the Embive interpreter.
As such, the transpilation can even be done ahead-of-time and by a different machine.

## Languages

To target Embive, a language/toolchain must have the following pre-requisites:
- Support RISC-V 32-bit bare-metal targets
- Allow custom linking (text at `0x00000000` and data at `0x80000000`)
- Output an ELF file (`.elf`)

Embive templates are available for the following languages:
- [C/C++](https://github.com/embive/embive-c-template)
- [Nim](https://github.com/embive/embive-nim-template)
- [Rust](https://github.com/embive/embive-rust-template)

## Example

```rust
use core::num::NonZeroI32;
use embive::{
    interpreter::{
        memory::{Memory, SliceMemory},
        registers::CPURegister, Error,
        Config, Interpreter, State, SYSCALL_ARGS,
    },
    transpiler::transpile_elf,
};

// RISC-V code to be transpiled and executed.
// The default code will execute the syscalls implemented
// bellow, loading values from RAM and adding them.
// Check the available Embive templates for more info.
const ELF_FILE: &[u8] = include_bytes!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/app.elf")
);

// A simple syscall implementation
fn syscall<M: Memory>(
    nr: i32,
    args: &[i32; SYSCALL_ARGS],
    memory: &mut M,
) -> Result<Result<i32, NonZeroI32>, Error> {
    // Match the syscall number
    let ret = match nr {
        // Add two numbers (arg[0] + arg[1])
        1 => Ok(args[0] + args[1]),
        // Load from RAM (arg[0])
        2 => match memory.load(args[0] as u32, 4) {
            Ok(val) => Ok(i32::from_le_bytes(val.try_into().unwrap())),
            Err(_) => Err(1.try_into().unwrap()), // Error loading
        },
        _ => Err(2.try_into().unwrap()), // Not implemented
    };

    // Outer Result is returned to the host (interpreter)
    // Inner result is returned to the guest code
    Ok(ret) // No host error
}

fn main() {
    // 16KB of code
    let mut code = [0; 16384];

    // Convert RISC-V ELF to Embive binary
    transpile_elf(ELF_FILE, &mut code).unwrap();

    // 4KB of RAM
    let mut ram = [0; 4096];

    // Create memory from code and RAM slices
    let mut memory = SliceMemory::new(&code, &mut ram);

    // Create interpreter config with instruction limit
    let config = Config::default()
        .with_instruction_limit(10);

    // Create interpreter
    let mut interpreter = Interpreter::new(&mut memory, config);

    // Run the interpreter, handling all possible states
    loop {
        match interpreter.run().unwrap() {
            // Keep running after reaching instruction limit
            State::Running => {},
            // Handle syscall if called by guest code
            State::Called => interpreter.syscall(&mut syscall).unwrap(),
            // Trigger an interrupt right-away if guest is waiting for it
            State::Waiting => interpreter.interrupt().unwrap(),
            // Stop if guest code exited
            State::Halted => break,
        }
    }

    // Code does "10 + 20" using syscalls (load from ram and add numbers)
    // Check the result (Ok(30)) (Registers: A0 = 0, A1 = 30)
    assert_eq!(
        interpreter
            .registers
            .cpu
            .get(CPURegister::A0 as u8)
            .unwrap(),
        0
    );
    assert_eq!(
        interpreter
            .registers
            .cpu
            .get(CPURegister::A1 as u8)
            .unwrap(),
        30
    );
}
```

## Instruction Limiting

In many cases, it is desirable to pause the guest after a number of instruction have been executed.

This can not only restrict the guest from using all the machine resources, but also allows the host
to execute other periodic tasks without relying on threading.

You can read more about instruction limiting in the `interpreter::Config` documentation.

## System Calls

System calls are a way for the interpreted code to interact with the host environment.  

When an `ecall` instruction is executed, the interpreter will return the state `Called` and
the host can then handle the syscall.  

You can read more about system calls in the `interpreter::Engine::syscall` documentation.

## Interrupts

Interrupts can be trigged on the guest code by the host. This is a complement to system calls,
allowing asynchronous half-duplex communication between the host and guest.  

When a `wfi` instruction is executed, the interpreter will return the state `Waiting`, meaning
that the guest has expressed that it is waiting for an interrupt to be triggered.  

You can read more about interrupts in the `interpreter::Engine::interrupt` documentation.

## Features

| Feature       | Default | Description                             | MSRV | Dependencies |
|---------------|---------|-----------------------------------------|------|--------------|
| `transpiler`  | ✅     | ELF-to-bytecode converter               | 1.81 | [elf](https://docs.rs/elf/latest/elf/)        |
| `interpreter` | ✅     | Execution engine                        | 1.81 | None         |
| `debugger`    | ❌     | Implement GDB Debugger for interpreter  | 1.81 | [gdbstub](https://github.com/daniel5151/gdbstub), [gdbstub_arch](https://github.com/daniel5151/gdbstub) |
| `alloc`       | ❌     | Transpilation without static buffer     | 1.81 | None         |
| `async`       | ❌     | Asynchronous syscall handling           | 1.85 | None         |

## Supported RISC-V Extensions

| Extension       | Status | Notes                          |
|-----------------|--------|--------------------------------|
| RV32I (Base)    | ✅     | Full compliance                |
| M (Multiply)    | ✅     | Hardware-accelerated           |
| A (Atomic)      | ✅     | LR/SC emulation                |
| C (Compressed)  | ✅     | 16-bit instruction support     |
| Zicsr           | ✅     | Machine CSRs implemented       |
| Zifencei        | ✅     | No-op in single-hart context   |

## What about Floating Point?

Rust doesn't support custom rounding modes nor does it expose the IEEE exception flags. Hence,
fully implementing the RISC-V F and/or D extensions would require a soft-float library.  

As at the time Embive was created no soft-float library satisfied my requirements (portable, safe, no_std, and complete),
it was decided to not support the floating point extensions.  

You can still use floats with Embive as the GCC/Clang compiler provides a soft-float implementation, but expect
larger binaries and slow performance. In some cases, you can offload the floating computation to the host using
syscalls.

## Minimum supported Rust version (MSRV)

Embive default features are guaranteed to compile on stable Rust 1.81 and up.  
Check the [Features section](#features) for more information.

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
