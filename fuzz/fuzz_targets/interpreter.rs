#![no_main]
use core::num::NonZeroI32;
use libfuzzer_sys::fuzz_target;

use embive::interpreter::{
    memory::{Memory, SliceMemory},
    Interpreter, State, SYSCALL_ARGS,
};

const MAX_INSTRUCTIONS: u32 = 2048;
const RAM_SIZE: usize = 256;

fn syscall<M: Memory>(
    _nr: i32,
    _args: &[i32; SYSCALL_ARGS],
    _memory: &mut M,
) -> Result<Result<i32, NonZeroI32>, ()> {
    Ok(Ok(0))
}

fuzz_target!(|data: &[u8]| {
    let mut ram = [0; RAM_SIZE];
    let mut memory = SliceMemory::new(data, &mut ram);
    let mut interpreter = Interpreter::new(&mut memory, MAX_INSTRUCTIONS);

    loop {
        match interpreter.run() {
            Ok(State::Called) => {
                if interpreter.syscall(&mut syscall).is_err() {
                    break;
                }
            }
            Ok(State::Waiting) => {
                if interpreter.interrupt(0).is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
});
