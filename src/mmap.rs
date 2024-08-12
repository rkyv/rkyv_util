//! Bindings for working with memory mapped objects in a cleaner way.

use memmap2::{Mmap, MmapMut};

use crate::owned::{StableBytes, StableBytesMut};



unsafe impl StableBytes for Mmap {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for MmapMut {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytesMut for MmapMut {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}



#[cfg(test)]
mod tests {
    use std::error::Error;



    #[test]
    pub fn to_owned_archive() -> Result<(), Box<dyn Error>> {


        Ok(())
    }
}
