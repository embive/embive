//! GDB TCP Server Example
//!
//! To use it, you must have installed `riscv32-unknown-elf-gdb` from the [RISC-V toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain).
//!
//! Example:
//! -> Run the example with `cargo run --example gdb_tcp <binary.elf>`
//! -> Connect to the gdb server with `riscv32-unknown-elf-gdb <binary.elf> -ex "target remote localhost:9001"`
//! -> Use gdb commands to debug the program (ex: `step`, `info registers`, `break`, etc)
//!
use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::num::NonZeroI32;

use embive::interpreter::memory::{Memory, SliceMemory};
use embive::interpreter::Debugger;
use embive::interpreter::Interpreter;
use embive::interpreter::SYSCALL_ARGS;
use embive::transpiler::transpile_elf;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::{DisconnectReason, GdbStub};

// A simple connection implementation for gdbstub using TcpStream.
struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl Connection for TcpConnection {
    type Error = std::io::Error;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        use std::io::Write;

        Write::write_all(&mut self.stream, &[byte])
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        use std::io::Write;

        Write::write_all(&mut self.stream, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        use std::io::Write;

        Write::flush(&mut self.stream)
    }

    fn on_session_start(&mut self) -> Result<(), Self::Error> {
        // see https://github.com/daniel5151/gdbstub/issues/28
        self.stream.set_nodelay(true)
    }
}

impl ConnectionExt for TcpConnection {
    fn read(&mut self) -> Result<u8, Self::Error> {
        use std::io::Read;

        self.stream.set_nonblocking(false)?;

        let mut buf = [0u8];
        match Read::read_exact(&mut self.stream, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }

    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        self.stream.set_nonblocking(true)?;

        let mut buf = [0u8];
        match self.stream.peek(&mut buf) {
            Ok(_) => Ok(Some(buf[0])),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
}

// A simple syscall implementation
fn syscall<M: Memory>(
    nr: i32,
    args: &[i32; SYSCALL_ARGS],
    memory: &mut M,
) -> Result<i32, NonZeroI32> {
    // Match the syscall number
    match nr {
        // Add two numbers (arg[0] + arg[1])
        1 => Ok(args[0] + args[1]),
        // Load from RAM (arg[0])
        2 => match memory.load(args[0] as u32, 4) {
            Ok(val) => Ok(i32::from_le_bytes(val.try_into().unwrap())),
            Err(_) => Err(1.try_into().unwrap()), // Error loading
        },
        _ => Err(2.try_into().unwrap()), // Not implemented
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the ELF file path from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <binary.elf>", args[0]);
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput).into());
    }

    // Read the ELF file
    println!("Reading ELF: {}", args[1]);
    let elf = std::fs::read(&args[1])?;

    // Transpile the ELF file
    let mut code = [0; 256 * 1024];
    println!("Transpiling ELF...");
    transpile_elf(&elf, &mut code)?;
    println!("ELF transpiled!");

    // Initialize the debugger
    let mut ram = [0; 64 * 1024];
    let mut memory = SliceMemory::new(code.as_slice(), &mut ram);
    let mut debugger: Debugger<'_, _, TcpConnection, _> = Debugger::new(&mut memory, syscall);

    // Wait for GDB client to connect
    println!("Waiting for GDB client to connect (localhost:9001)...");
    let sock = TcpListener::bind(SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9001))?;
    let (stream, addr) = sock.accept()?;
    let conn = TcpConnection::new(stream);
    println!("Connected to: {}", addr);

    // Create a GDB server
    let mut buffer = [0; 4096];
    let gdb = GdbStub::builder(conn)
        .with_packet_buffer(&mut buffer)
        .build()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Run the GDB server
    match gdb.run_blocking::<Debugger<'_, _, TcpConnection, _>>(&mut debugger) {
        Ok(disconnect_reason) => match disconnect_reason {
            DisconnectReason::Disconnect => {
                println!("GDB client has disconnected.");
            }
            DisconnectReason::TargetExited(code) => {
                println!("Target exited with code {}!", code)
            }
            DisconnectReason::TargetTerminated(sig) => {
                println!("Target terminated with signal {}!", sig)
            }
            DisconnectReason::Kill => println!("GDB sent a kill command!"),
        },
        Err(e) => {
            if e.is_target_error() {
                println!(
                    "target encountered a fatal error: {}",
                    e.into_target_error().unwrap()
                )
            } else if e.is_connection_error() {
                let (e, kind) = e.into_connection_error().unwrap();
                println!("connection error: {:?} - {}", kind, e,)
            } else {
                println!("gdbstub encountered a fatal error: {}", e)
            }
        }
    }

    // Interpreter state after debugging ended
    let interpreter: Interpreter<'_, SliceMemory> = debugger.into();
    println!("");
    println!("Interpreter State:");
    println!("Registers: {:?}", interpreter.registers.cpu);
    println!("Program Counter: {:#08x}", interpreter.program_counter);
    println!("Instruction: {:?}", interpreter.fetch());

    return Ok(());
}
