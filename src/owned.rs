//! Enables us to pass around owned archive types.

use std::{marker::PhantomData, ops::Deref};

use rkyv::Archive;


/// An owned archive type.
#[derive(Default)]
pub struct OwnedArchive<T, C: StableBytes>

{
    container: C,
    _type: PhantomData<T>
}

impl<T, C: StableBytes> OwnedArchive<T, C> {
    /// Creates a new OwnedArchive
    pub fn new<E>(container: C) -> Result<Self, E>
        //where 
         //   T: CheckBytes<Bro>
    {
        
        
        Ok(Self {
            container,
            _type: PhantomData
        })
    }
}

impl<C: StableBytes, T: Archive> Deref for OwnedArchive<T, C> {
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { rkyv::access_unchecked(self.container.bytes()) }
    }
}


// TODO: Implement clone and debug conditionally



/// An interface to access stable bytes.
///
/// The value of bytes should never change between
/// accesses.
///
/// # Safety
/// The interface simply requires that the byte value never
/// changes.
///
/// For instance, the following malicious implementation would be unsafe:
/// ```
/// struct Malicious {
///     counter: RefCell<u8>
/// }
///
/// unsafe impl StableBytes for Malicious {
///     fn bytes(&self) -> &[u8] {
///         counter.borrow_mut() += 1;
///         if *counter % 2 == 0 {
///             [0x00]
///         } else {
///             [0x01]
///         }
///     }
///
/// }
/// ```
/// The above code does not meet the safety contract because
/// every other access will return a different set of bytes.
///
/// Another example for safety is as follows:
/// ```
/// struct Good {
///     data: Vec<u8>
/// }
///
/// unsafe impl StableBytes for Good {
///     fn bytes(&self) -> &[u8] {
///         self.data.as_ref()
///     }
/// }
/// ```
/// The above implementation is only safe if the `Vec<u8>` inside
/// of `Good` is never mutated. Otherwise the bytes could be changed.
pub unsafe trait StableBytes {
    /// Gets the underlying bytes.
    fn bytes(&self) -> &[u8];
}

#[cfg(test)]
mod tests {

    
}
