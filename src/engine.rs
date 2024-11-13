//! Virtual RISC-V engine.
//! A simple virtual RISC-V engine that can run RISC-V bytecode.

use core::fmt::Debug;
use crate::error::EmbiveError;
use crate::instruction::decode_and_execute;

/// Number of registers in the virtual RISC-V engine.
pub(crate) const REGISTER_COUNT: usize = 32;

/// Memory address offset for the virtual RISC-V engine.
pub(crate) const MEMORY_OFFSET: i32 = 0x20000000;

/// Stack pointer register index.
pub(crate) const SP: usize = 2;

/// Generate initial register values.
/// The stack pointer (x2) is set to the end of memory, and all other registers are set to 0.
fn initial_registers(memory_size: usize) -> [i32; REGISTER_COUNT] {
    let mut registers = [0; REGISTER_COUNT];

    // Set the stack pointer to the top of the stack
    registers[SP] = MEMORY_OFFSET + memory_size as i32;
    registers
}

/// The virtual RISC-V engine.
pub struct Engine<'a> {
    pc: i32,
    registers: [i32; REGISTER_COUNT],
    memory: &'a mut [u8],
    program: &'a [u8],
}

impl Debug for Engine<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Engine")
            .field("pc", &self.pc)
            .field("registers", &self.registers)
            .field("memory", &self.memory.len())
            .field("program", &self.program.len())
            .finish()
    }
}

impl<'a> Engine<'a> {
    /// Create a new virtual RISC-V engine.
    ///
    /// Arguments:
    /// - `program`: Program to run, `u8` slice. RISC-V bytecode mapped to address 0x00000000 of the virtual processor.
    /// - `memory`: Memory buffer, mutable `u8` slice. Internally mapped to address `MEMORY_OFFSET` of the virtual processor.
    pub fn new(program: &'a [u8], memory: &'a mut [u8]) -> Result<Engine<'a>, EmbiveError> {
        Ok(Engine {
            pc: 0,
            registers: initial_registers(memory.len()),
            memory,
            program,
        })
    }

    /// Reset the virtual RISC-V engine:
    /// - Program counter is reset to 0.
    /// - Registers are reset to 0.
    /// - Stack pointer (x2) is set to the top of the stack.
    pub fn reset(&mut self) {
        self.pc = 0;
        self.registers = initial_registers(self.memory.len());
    }

    /// Run the virtual RISC-V engine from the start, until a halt instruction is reached.
    /// The engine is reset before execution (by implicitly calling the `reset` method).
    ///
    /// Returns:
    /// - `Ok`: Executed successfully (halt was reached).
    /// - `Err(EmbiveError)`: Failed to execute.
    pub fn run(&mut self) -> Result<(), EmbiveError> {
        self.reset();

        loop {
            // Step through the program
            if !self.step()? {
                // Halt execution
                return Ok(());
            }
        }
    }

    /// Run a single instruction from the virtual RISC-V engine.
    ///
    /// Returns:
    /// - `Ok(bool)`: Executed successfully.
    ///     - `True`: Should continue execution.
    ///     - `False`: Should halt.
    /// - `Err(EmbiveError)`: Failed to execute.
    pub fn step(&mut self) -> Result<bool, EmbiveError> {
        // Fetch next instruction
        let data = self.fetch()?;

        // Decode and execute the instruction
        decode_and_execute(self, data)
    }

    /// Read an engine register.
    ///
    /// Arguments:
    /// - `index`: The index of the register (from 0 to `REGISTER_COUNT`).
    ///
    /// Returns:
    /// - `Ok(i32)`: The value of the register.
    /// - `Err(EmbiveError)`: The register index is out of bounds.
    pub fn register(&self, index: u8) -> Result<i32, EmbiveError> {
        self.registers
            .get(index as usize)
            .copied()
            .ok_or(EmbiveError::InvalidRegister)
    }

    /// Get a mutable reference to an engine register.
    ///
    /// Arguments:
    /// - `index`: The index of the register (from 1 to `REGISTER_COUNT`).
    ///     - Register 0 is hardwired to 0, so it is not mutable.
    ///
    /// Returns:
    /// - `Ok(&mut i32)`: Mutable reference to the register.
    /// - `Err(EmbiveError)`: The register index is out of bounds.
    pub fn register_mut(&mut self, index: u8) -> Result<&mut i32, EmbiveError> {
        // Register 0 is hardwired to 0
        self.registers[1..REGISTER_COUNT]
            .get_mut(index as usize - 1)
            .ok_or(EmbiveError::InvalidRegister)
    }

    /// Get a mutable reference to the program counter.
    ///
    /// Returns:
    /// - `&mut i32`: Mutable reference to the program counter.
    pub fn pc_mut(&mut self) -> &mut i32 {
        &mut self.pc
    }

    /// Load `N` bytes from the memory address.
    ///
    /// Arguments:
    /// - `address`: The memory address to get.
    ///     - The address is offset by `MEMORY_OFFSET`.
    ///
    /// Returns:
    /// - `Ok([u8; N])`: The bytes at the memory address.
    /// - `Err(EmbiveError)`: The memory address and/or `N` are out of bounds.
    pub fn load<const N: usize>(&self, address: i32) -> Result<[u8; N], EmbiveError> {
        let data;
        if address < MEMORY_OFFSET {
            data = self
                .program
                .get(address as usize..address as usize + N)
                .ok_or(EmbiveError::InvalidMemoryAddress)?;
        } else {
            let address = address - MEMORY_OFFSET;
            data = self
                .memory
                .get(address as usize..address as usize + N)
                .ok_or(EmbiveError::InvalidMemoryAddress)?;
        }

        // Unwrap is safe because the slice is guaranteed to have N elements.
        Ok(data.try_into().unwrap())
    }

    /// Store `N` bytes to the memory address.
    ///
    /// Arguments:
    /// - `address`: The memory address to store.
    ///    - The address is offset by `MEMORY_OFFSET`.
    /// - `data`: The bytes to store.
    ///
    /// Returns:
    /// - `Ok(())`: The bytes were stored successfully.
    /// - `Err(EmbiveError)`: The memory address and/or `N` are out of bounds.
    pub fn store<const N: usize>(
        &mut self,
        address: i32,
        data: [u8; N],
    ) -> Result<(), EmbiveError> {
        let address = address - MEMORY_OFFSET;
        self.memory
            .get_mut(address as usize..address as usize + N)
            .ok_or(EmbiveError::InvalidMemoryAddress)?
            // copy_from_slice is safe because the slice is guaranteed to have N elements.
            .copy_from_slice(&data);

        Ok(())
    }

    /// Fetch the next instruction (raw) from the program counter.
    ///
    /// Returns:
    /// - `Ok(u32)`: The instruction (raw) that was fetched.
    /// - `Err(EmbiveError)`: The program counter is out of bounds.
    pub fn fetch(&mut self) -> Result<u32, EmbiveError> {
        let inst = self
            .program
            .get(self.pc as usize..self.pc as usize + 4)
            .ok_or(EmbiveError::InvalidProgramCounter)?;
        Ok(u32::from_le_bytes(inst.try_into().unwrap()))
    }
}
