//! Memory Module

use core::fmt::Debug;

use super::error::Error;

/// RAM address offset
pub const RAM_OFFSET: u32 = 0x80000000;

/// Embive Memory Trait
/// This trait implements the memory interface for the Embive interpreter.
/// It should support loading bytes from the code (0x00000000) region, as well as loading and storing to the RAM ([`RAM_OFFSET`]).
/// RISC-V is little-endian, bytes should be loaded / stored as that.
pub trait Memory {
    /// Load `len` bytes from memory address.
    /// Memory address can be from code (0x00000000) or RAM ([`RAM_OFFSET`]) region.
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: Memory address to get (code or RAM).
    /// - `len`: Number of bytes to load.
    ///
    /// Returns:
    /// - `Ok(&[u8])`: Bytes at the memory address.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn load(&self, address: u32, len: u32) -> Result<&[u8], Error>;

    /// Store `len` bytes to memory address.
    /// Memory address can only be from RAM ([`RAM_OFFSET`]) region.
    /// RISC-V is little-endian, always use `to_le_bytes()` and `from_le_bytes()`.
    ///
    /// Arguments:
    /// - `address`: The memory address to store (only RAM).
    /// - `data`: Bytes to store.
    ///
    /// Returns:
    /// - `Ok(())`: Bytes were stored successfully.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn store(&mut self, address: u32, data: &[u8]) -> Result<(), Error>;
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
    #[inline]
    fn load(&self, address: u32, len: u32) -> Result<&[u8], Error> {
        // Check if the address is in RAM or code.
        if address >= RAM_OFFSET {
            // Subtract the RAM offset to get the actual address.
            let ram_address = address.wrapping_sub(RAM_OFFSET);

            self.ram
                .get(ram_address as usize..(ram_address + len) as usize)
                .ok_or(Error::InvalidMemoryAddress(address))
        } else {
            self.code
                .get(address as usize..(address + len) as usize)
                .ok_or(Error::InvalidMemoryAddress(address))
        }
    }

    #[inline]
    fn store(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        // Subtract the RAM offset to get the actual address.
        let ram_address = address.wrapping_sub(RAM_OFFSET);

        let ram_slice = self
            .ram
            .get_mut(ram_address as usize..(ram_address as usize + data.len()))
            .ok_or(Error::InvalidMemoryAddress(address))?;
        ram_slice.copy_from_slice(data);

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
        let result = memory.load(0x80000000, 4);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn load_out_of_ram() {
        let mut ram = [0; 2];
        let memory = SliceMemory::new(&[], &mut ram);
        let result = memory.load(0x80000000, 4);

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
        let result = memory.store(0x80000000, &[0x1, 0x2, 0x3, 0x4]);

        assert!(result.is_ok());
        assert_eq!(ram, [0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn store_out_of_ram() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let result = memory.store(0x80000000, &[0; 4]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }

    #[test]
    pub fn load_code() {
        let code = [0x1, 0x2, 0x3, 0x4];
        let memory = SliceMemory::new(&code, &mut []);
        let result = memory.load(0x0, 4);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0x1, 0x2, 0x3, 0x4]);
    }

    #[test]
    pub fn load_out_of_code() {
        let code = [0; 2];
        let memory = SliceMemory::new(&code, &mut []);
        let result = memory.load(0x0, 4);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidMemoryAddress(_)
        ));
    }
}
