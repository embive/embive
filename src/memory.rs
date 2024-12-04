//! Memory Module

use crate::error::EmbiveError;
use core::fmt::Debug;

/// RAM address offset
pub const RAM_OFFSET: u32 = 0x80000000;

/// Embive Memory Trait
/// This trait implements the memory interface for the Embive engine.
/// It should support loading bytes from the code (0x0x00000000) region, as well as loading and storing to the RAM ([`RAM_OFFSET`]).
/// RISC-V is little-endian, bytes should be loaded / stored as that.
pub trait Memory {
    /// Load `N` bytes from memory address.
    /// Memory address can be from code (0x0x00000000) or RAM ([`RAM_OFFSET`]) region.
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: Memory address to get (code or RAM).
    ///
    /// Returns:
    /// - `Ok([u8; N])`: Bytes at the memory address.
    /// - `Err(EmbiveError)`: An error occurred. Ex.: Memory address is out of bounds.
    fn load<const N: usize>(&self, address: u32) -> Result<[u8; N], EmbiveError>;

    /// Store `N` bytes to memory address.
    /// Memory address can only be from RAM ([`RAM_OFFSET`]) region.
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: The memory address to store (only RAM).
    /// - `data`: Bytes to store.
    ///
    /// Returns:
    /// - `Ok(())`: Bytes were stored successfully.
    /// - `Err(EmbiveError)`: An error occurred. Ex.: Memory address is out of bounds.
    fn store<const N: usize>(&mut self, address: u32, data: [u8; N]) -> Result<(), EmbiveError>;
}

/// A simple memory implementation using slices.
/// This memory implementation is used to create a memory space from code and RAM slices.
#[derive(Debug)]
pub struct SliceMemory<'a> {
    /// RISC-V bytecode.
    code: &'a [u8],
    /// RAM buffer.
    ram: &'a mut [u8],
}

impl SliceMemory<'_> {
    /// Create a new memory space.
    ///
    /// Arguments:
    /// - `code`: Code buffer, `u8` slice.
    /// - `ram`: RAM buffer, mutable `u8` slice.
    pub fn new<'a>(code: &'a [u8], ram: &'a mut [u8]) -> SliceMemory<'a> {
        SliceMemory { code, ram }
    }
}

impl Memory for SliceMemory<'_> {
    fn load<const N: usize>(&self, address: u32) -> Result<[u8; N], EmbiveError> {
        // Check if the address is in RAM or code.
        if address >= RAM_OFFSET {
            // Subtract the RAM offset to get the actual address.
            let address = address - RAM_OFFSET;

            if (address as usize + N) > self.ram.len() {
                return Err(EmbiveError::InvalidMemoryAddress);
            }

            // Unwrap is safe because the slice is guaranteed to at least have N elements.
            Ok(*self.ram[address as usize..].first_chunk::<N>().unwrap())
        } else {
            if (address as usize + N) > self.code.len() {
                return Err(EmbiveError::InvalidMemoryAddress);
            }

            // Unwrap is safe because the slice is guaranteed to at least have N elements.
            Ok(*self.code[address as usize..].first_chunk::<N>().unwrap())
        }
    }

    fn store<const N: usize>(&mut self, address: u32, data: [u8; N]) -> Result<(), EmbiveError> {
        let address = address.wrapping_sub(RAM_OFFSET);

        if (address as usize + N) > self.ram.len() {
            return Err(EmbiveError::InvalidMemoryAddress);
        }

        // Unwrap is safe because the slice is guaranteed to have at least N elements.
        *self.ram[address as usize..].first_chunk_mut::<N>().unwrap() = data;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn load_ram() {
        let mut ram = [0x1, 0x2, 0x3, 0x4];
        let memory = SliceMemory::new(&[], &mut ram);
        let result = memory.load::<4>(0x80000000);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn load_out_of_ram() {
        let mut ram = [0; 2];
        let memory = SliceMemory::new(&[], &mut ram);
        let result = memory.load::<4>(0x80000000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EmbiveError::InvalidMemoryAddress);
    }

    #[test]
    pub fn store_ram() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.store::<4>(0x80000000, [0x1, 0x2, 0x3, 0x4]);

        assert!(result.is_ok());
        assert_eq!(ram, [0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn store_out_of_ram() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.store::<4>(0x80000000, [0; 4]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EmbiveError::InvalidMemoryAddress);
    }

    #[test]
    pub fn load_code() {
        let code = [0x1, 0x2, 0x3, 0x4];
        let memory = SliceMemory::new(&code, &mut []);
        let result = memory.load::<4>(0x0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn load_out_of_code() {
        let code = [0; 2];
        let memory = SliceMemory::new(&code, &mut []);
        let result = memory.load::<4>(0x0);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EmbiveError::InvalidMemoryAddress);
    }
}
