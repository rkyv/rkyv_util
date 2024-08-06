//! Enables us to pass around owned archive types.
//!
//! This can be useful in many situations. For instance, suppose
//! we want to pass Archives around in channels but we do not want
//! to deal with complicated lifetimes.

use core::fmt::Debug;
use std::{marker::PhantomData, ops::Deref, rc::Rc, sync::Arc};
use rkyv::{api::high::HighValidator, bytecheck::CheckBytes, util::AlignedVec, Archive, Portable};


/// An owned archive type.
#[derive(Default)]
pub struct OwnedArchive<T, C> {
    container: C,
    _type: PhantomData<T>
}

impl<T, C> OwnedArchive<T, C> {
    /// Creates a new OwnedArchive
    pub fn new<E>(container: C) -> Result<Self, E>
    where 
         //T: CheckBytes<Strategy<Validator<ArchiveValidator<'_>, SharedValidator>>> + Portable,
        T: Portable + for<'a> CheckBytes<HighValidator<'a, E>>, 
        E: rkyv::rancor::Source,
        C: StableBytes
    {

        rkyv::access::<T, E>(container.bytes())?;
        
        
        Ok(Self {
            container,
            _type: PhantomData
        })
    }
}

impl<C: StableBytes, T: Archive> Deref for OwnedArchive<T, C> {
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        // # Safety
        // Here we can safely access the underlying archive. This is
        // because `StableBytes` enforces the safety contract that the underlying
        // bytes remain stable, and thus the container that we took ownership of
        // when creating the `OwnedArchive` has already been created.
        unsafe { rkyv::access_unchecked(self.container.bytes()) }
    }
}


// TODO: Implement clone and debug conditionally
impl<T, C: Clone> Clone for OwnedArchive<T, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            _type: self._type
        }
    }
}


impl<T: Archive, C: StableBytes> Debug for OwnedArchive<T, C>
where 
    T::Archived: Debug 
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}


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
/// use rkyv_util::owned::StableBytes;
/// use std::cell::RefCell;
///
/// struct Malicious {
///     counter: RefCell<u8>
/// }
///
/// unsafe impl StableBytes for Malicious {
///     fn bytes(&self) -> &[u8] {
///         *self.counter.borrow_mut() += 1;
///         if *self.counter.borrow() % 2 == 0 {
///             &[0x00]
///         } else {
///             &[0x01]
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
/// use rkyv_util::owned::StableBytes;
/// 
///
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

unsafe impl StableBytes for AlignedVec {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for Vec<u8> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for Arc<[u8]> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for Rc<[u8]> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for Box<[u8]> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_owned_archive_vec() {

    }
    
}
