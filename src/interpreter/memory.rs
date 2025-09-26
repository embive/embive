//! Memory Module
//!
//! This module implements the memory interface for the Embive interpreter.
mod memory_type;

use core::{fmt::Debug, ops::Range};

use crate::interpreter::utils::unlikely;

use super::error::Error;

#[doc(inline)]
pub use memory_type::MemoryType;

/// RAM address offset for default memory implementations.
pub const RAM_OFFSET: u32 = 0x80000000;

/// A helper function to check if a slice range is valid.
///
/// Arguments:
/// - `slice`: The slice to check.
/// - `start`: The start index of the range.
/// - `len`: The length of the range.
///
/// Returns:
/// - `Ok(Range<usize>)`: The valid range.
/// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
#[inline(always)]
fn checked_slice_range(slice: &[u8], start: usize, len: usize) -> Result<Range<usize>, Error> {
    // Check for overflow when calculating the end index.
    let end = start
        .checked_add(len)
        .ok_or(Error::InvalidMemoryAccessLength(len))?;

    // Check bounds, start is always <= end here.
    if unlikely(end > slice.len()) {
        return Err(Error::InvalidMemoryAddress(end as u32));
    }

    Ok(start..end)
}

/// Embive Memory Trait
///
/// This trait implements the memory interface for the Embive interpreter.
/// It should support loading bytes from the code (0x00000000) region, as well as loading and storing to the RAM ([`RAM_OFFSET`]).
/// RISC-V is little-endian, bytes should be loaded / stored as that.
pub trait Memory {
    /// Load `len` bytes from memory address.
    ///
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: Memory address to get (code or RAM).
    /// - `len`: Number of bytes to load.
    ///
    /// Returns:
    /// - `Ok(&[u8])`: Bytes at the memory address.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn load_bytes(&mut self, address: u32, len: usize) -> Result<&[u8], Error>;

    /// Get mutable reference to `len` bytes from memory address.
    ///
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: Memory address to get (only RAM).
    /// - `len`: Number of bytes to get.
    ///
    /// Returns:
    /// - `Ok(&mut [u8])`: Mutable bytes at the memory address.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn mut_bytes(&mut self, address: u32, len: usize) -> Result<&mut [u8], Error>;

    /// Store `len` bytes to memory address.
    ///
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: The memory address to store (only RAM).
    /// - `data`: Bytes to store.
    ///
    /// Returns:
    /// - `Ok(())`: Bytes were stored successfully.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn store_bytes(&mut self, address: u32, data: &[u8]) -> Result<(), Error>;
}

/// A simple memory implementation using slices.
///
/// This memory implementation creates a memory space from code and RAM slices.
///
/// Code section is mapped to address `0x00000000` and RAM to [`RAM_OFFSET`].
#[derive(Debug)]
pub struct SliceMemory<'a> {
    /// RISC-V bytecode.
    code: &'a [u8],
    /// RAM buffer.
    ram: &'a mut [u8],
}

impl<'a> SliceMemory<'a> {
    /// Create a new memory space.
    ///
    /// Arguments:
    /// - `code`: Code buffer, `u8` slice.
    /// - `ram`: RAM buffer, mutable `u8` slice.
    pub fn new(code: &'a [u8], ram: &'a mut [u8]) -> SliceMemory<'a> {
        SliceMemory { code, ram }
    }
}

impl Memory for SliceMemory<'_> {
    #[inline]
    fn load_bytes(&mut self, address: u32, len: usize) -> Result<&[u8], Error> {
        // Check if the address is in RAM or code.
        if address >= RAM_OFFSET {
            // Subtract the RAM offset to get the actual address.
            let ram_address = address.wrapping_sub(RAM_OFFSET) as usize;
            checked_slice_range(self.ram, ram_address, len).map(|r| &self.ram[r])
        } else {
            let code_address = address as usize;
            checked_slice_range(self.code, code_address, len).map(|r| &self.code[r])
        }
    }

    #[inline]
    fn mut_bytes(&mut self, address: u32, len: usize) -> Result<&mut [u8], Error> {
        // Subtract the RAM offset to get the actual address.
        let ram_address = address.wrapping_sub(RAM_OFFSET) as usize;
        checked_slice_range(self.ram, ram_address, len).map(|r| &mut self.ram[r])
    }

    #[inline]
    fn store_bytes(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        // Subtract the RAM offset to get the actual address.
        let ram_address = address.wrapping_sub(RAM_OFFSET) as usize;
        checked_slice_range(self.ram, ram_address, data.len()).map(|r| {
            self.ram[r].copy_from_slice(data);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn load_ram() {
        let mut ram = [0x1, 0x2, 0x3, 0x4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.load_bytes(0x80000000, 4);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn mut_ram() {
        let mut ram = [0x1, 0x2, 0x3, 0x4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.mut_bytes(0x80000000, 4);

        assert!(result.is_ok());

        let bytes = result.unwrap();
        bytes[0] = 0x5;

        assert_eq!(bytes, &mut [0x5, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn load_out_of_ram() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.load_bytes(0x80000000, 4);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn mut_out_of_ram() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.mut_bytes(0x80000000, 4);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn store_ram() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.store_bytes(0x80000000, &[0x1, 0x2, 0x3, 0x4]);

        assert!(result.is_ok());
        assert_eq!(ram, [0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn store_out_of_ram() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.store_bytes(0x80000000, &[0; 4]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn load_code() {
        let code = [0x1, 0x2, 0x3, 0x4];
        let mut memory = SliceMemory::new(&code, &mut []);
        let result = memory.load_bytes(0x0, 4);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn mut_code() {
        let code = [0; 2];
        let mut memory = SliceMemory::new(&code, &mut []);
        let result = memory.mut_bytes(0x0, 4);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn store_code() {
        let code = [0; 4];
        let mut memory = SliceMemory::new(&code, &mut []);
        let result = memory.store_bytes(0x0, &[0x1, 0x2, 0x3, 0x4]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn load_out_of_code() {
        let code = [0; 2];
        let mut memory = SliceMemory::new(&code, &mut []);
        let result = memory.load_bytes(0x0, 4);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }
}
