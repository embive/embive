use embassy_executor::Spawner;
use embassy_futures::yield_now;
use log::*;

use core::num::NonZeroI32;
use embive::{
    interpreter::{
        memory::{Memory, SliceMemory},
        registers::CPURegister,
        Config, Interpreter, State, SYSCALL_ARGS,
    },
    transpiler::transpile_elf,
};

// RISC-V code to transpile and execute
const ELF_FILE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/app.elf"));

// A simple async syscall implementation
async fn syscall<M: Memory>(
    nr: i32,
    args: &[i32; SYSCALL_ARGS],
    memory: &mut M,
) -> Result<Result<i32, NonZeroI32>, ()> {
    info!("Entering syscall: {}", nr);
    yield_now().await; // Simulate async syscall delay
    info!("Args: {:?}", args);

    // Match the syscall number (always succeeds)
    Ok(match nr {
        // Add two numbers (arg[0] + arg[1])
        1 => Ok(args[0] + args[1]),
        // Load from RAM (arg[0])
        2 => match memory.load(args[0] as u32, 4) {
            Ok(val) => Ok(i32::from_le_bytes(val.try_into().unwrap())),
            Err(_) => Err(1.try_into().unwrap()), // Error loading
        },
        _ => Err(2.try_into().unwrap()), // Not implemented
    })
}

#[embassy_executor::task]
async fn run() {
    loop {
        info!("tick");
        yield_now().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    spawner.spawn(run()).unwrap();

    // 16KB of code
    let mut code = [0; 16384];

    // Convert RISC-V ELF to Embive binary
    transpile_elf(ELF_FILE, &mut code).unwrap();

    // 4KB of RAM
    let mut ram = [0; 4096];

    // Create memory from code and RAM slices
    let mut memory = SliceMemory::new(&code, &mut ram);

    // Create interpreter config
    let config = Config::default().with_instruction_limit(10);

    // Create interpreter
    let mut interpreter = Interpreter::new(&mut memory, config);

    // Run it until ebreak, triggering an interrupt after every wfi
    loop {
        match interpreter.run().unwrap() {
            State::Running => {
                // Yield to other tasks after instruction limit
                info!("Yielding...");
                yield_now().await;
            }
            State::Called => interpreter.syscall_async(&mut syscall).await.unwrap(),
            State::Waiting => interpreter.interrupt(0).unwrap(),
            State::Halted => break,
        }
    }

    // Code does "10 + 20" using syscalls (load from ram and add numbers)
    // Check the result (Ok(30)) (A0 = 0, A1 = 30)
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

    info!("Interpreter halted!");
    std::process::exit(0);
}
