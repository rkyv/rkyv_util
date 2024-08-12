//! Enables us to pass around owned archive types.
//!
//! This can be useful in many situations. For instance, suppose
//! we want to pass Archives around in channels but we do not want
//! to deal with complicated lifetimes.

use core::fmt::Debug;
use core::{marker::PhantomData, ops::Deref, pin::Pin};

//use memmap2::{Mmap, MmapMut};
use rkyv::{
    api::high::HighValidator, bytecheck::CheckBytes, rancor::Source, util::AlignedVec, Archive, Portable
};

/// An owned archive type.
///
/// This requires a container that implements the `StableBytes`
/// interface according to it's contract. On creation, it will attempt
/// to deserialize and check the bytes. If this succeeds, we will have
/// created an `OwnedArchive`.
///
/// Dereferences will directly pointer cast to the archive, allowing
/// quick access to the underlying archive.
///
/// # Example
/// Creating an owned archive from bytes.
/// ```
/// use rkyv::{rancor::Error, util::AlignedVec};
/// use rkyv_util::owned::OwnedArchive;
///
/// #[derive(rkyv::Archive, rkyv::Serialize)]
/// #[rkyv(check_bytes)]
/// pub struct Test {
///     hello: u8,
/// }
///
/// let bytes = rkyv::to_bytes::<Error>(&Test { hello: 2 }).unwrap();
///
/// let owned_archive = OwnedArchive::<Test, _>::new::<Error>(bytes).unwrap();
/// assert_eq!(owned_archive.hello, 2);
/// ```
#[derive(Default)]
pub struct OwnedArchive<T, C> {
    /// The container representing the bytes of our archive.
    container: C,
    /// The type that our archive will decompose into.
    _type: PhantomData<T>,
}

impl<T, C> OwnedArchive<T, C> {
    /// Creates a new `OwnedArchive` from a container
    /// that supports the `StableBytes` interface.
    pub fn new<E>(container: C) -> Result<Self, E>
    where
        T: Archive,
        T::Archived: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
        E: Source,
        C: StableBytes,
    {
        // Here we check if the bytes are good. If so, we will
        // allow for the creation of the `OwnedArchive`.
        rkyv::access::<T::Archived, E>(container.bytes())?;

        Ok(Self {
            container,
            _type: PhantomData,
        })
    }
    /// Gets the pinned object as mutable.
    ///
    /// # Example
    /// ```
    /// use rkyv::{rancor::Error, util::AlignedVec};
    /// use rkyv_util::owned::OwnedArchive;
    ///
    /// #[derive(rkyv::Archive, rkyv::Serialize)]
    /// #[rkyv(check_bytes)]
    /// pub struct Test {
    ///     hello: u8,
    /// }
    ///
    /// let bytes = rkyv::to_bytes::<Error>(&Test { hello: 2 }).unwrap();
    ///
    /// let owned_archive =
    ///     &mut OwnedArchive::<Test, _>::new::<Error>(bytes).unwrap();
    /// assert_eq!(owned_archive.hello, 2);
    ///
    /// owned_archive.get_mut().hello = 3;
    ///
    /// // `hello` should be 3.
    /// assert_eq!(owned_archive.hello, 3);
    /// ```
    pub fn get_mut(&mut self) -> Pin<&mut T::Archived>
    where
        T: Archive,
        T::Archived: Portable,
        C: StableBytesMut,
    {
        // # Safety
        // Here we can safely access the underlying archive. This is
        // because `StableBytesMut` enforces the safety contract that the
        // underlying bytes remain stable, and thus the container that
        // we took ownership of when creating the `OwnedArchive` has
        // already been created.
        unsafe {
            rkyv::access_unchecked_mut::<T::Archived>(
                self.container.bytes_mut(),
            )
        }
    }
}

impl<C: StableBytes, T: Archive> Deref for OwnedArchive<T, C> {
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        // # Safety
        // Here we can safely access the underlying archive. This is
        // because `StableBytes` enforces the safety contract that the
        // underlying bytes remain stable, and thus the container that
        // we took ownership of when creating the `OwnedArchive` has
        // already been created.
        unsafe { rkyv::access_unchecked(self.container.bytes()) }
    }
}

impl<T, C: Clone> Clone for OwnedArchive<T, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            _type: self._type,
        }
    }
}

impl<T: Archive, C: StableBytes> Debug for OwnedArchive<T, C>
where
    T::Archived: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}

/// A contract guaranteeing that bytes should originate
/// from the same source between accesses.
///
/// # Safety
/// - `bytes` always refers to the same buffer.
/// - The **only** way to edit the underlying bytes is through `StableBytesMut`
///
/// For instance, the following malicious implementation would be unsafe:
/// ```
/// use std::cell::RefCell;
///
/// use rkyv_util::owned::StableBytes;
///
/// struct Malicious {
///     counter: RefCell<u8>,
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
/// }
/// ```
/// The above code does not meet the safety contract because
/// every other access will return a different set of bytes and it is not
/// referencing a constant buffer.
///
/// Another example for possible unsafety is as follows:
/// ```
/// use rkyv_util::owned::StableBytes;
///
/// struct Good {
///     data: Vec<u8>,
/// }
///
/// unsafe impl StableBytes for Good {
///     fn bytes(&self) -> &[u8] {
///         self.data.as_ref()
///     }
/// }
/// ```
/// The above implementation is only safe if the `Vec<u8>` inside
/// of `Good` is only ever mutated through `StableBytesMut`.
/// Otherwise the bytes could be changed.
///
/// An example that is always safe would be the following:
/// ```
/// use std::sync::Arc;
///
/// use rkyv_util::owned::StableBytes;
///
/// struct Good {
///     data: Arc<[u8]>,
/// }
///
/// unsafe impl StableBytes for Good {
///     fn bytes(&self) -> &[u8] {
///         self.data.as_ref()
///     }
/// }
/// ```
/// Since the data within the `Arc<[u8]>` is not mutable, we can never change
/// it and thus this is necessarily stable.
pub unsafe trait StableBytes {
    /// Gets the underlying bytes.
    fn bytes(&self) -> &[u8];
}

/// A contract guaranteeing that bytes should originate
/// from the same source between acceses.
///
/// # Safety
/// - `bytes` always refers to the same buffer.
/// - The **only** way to edit the underlying bytes is through `StableBytesMut`
///
/// For instance, the following malicious implementation would be unsafe:
/// ```
/// use std::cell::RefCell;
///
/// use rkyv_util::owned::{StableBytes, StableBytesMut};
///
/// struct Malicious {
///     counter: RefCell<u8>,
///     buf_one: Vec<u8>,
///     buf_two: Vec<u8>,
/// }
///
/// unsafe impl StableBytes for Malicious {
///     fn bytes(&self) -> &[u8] {
///         self.buf_one.as_ref()
///     }
/// }
///
/// unsafe impl StableBytesMut for Malicious {
///     fn bytes_mut(&mut self) -> &mut [u8] {
///         *self.counter.borrow_mut() += 1;
///         if *self.counter.borrow() % 2 == 0 {
///             &mut self.buf_one
///         } else {
///             &mut self.buf_two
///         }
///     }
/// }
/// ```
/// The above code does not meet the safety contract because
/// every other access will return a different set of bytes and it is not
/// referencing a constant buffer.
///
/// Another example for possible unsafety is as follows:
/// ```
/// use rkyv_util::owned::{StableBytes, StableBytesMut};
///
/// struct Good {
///     data: Vec<u8>,
/// }
///
/// unsafe impl StableBytes for Good {
///     fn bytes(&self) -> &[u8] {
///         self.data.as_ref()
///     }
/// }
///
/// unsafe impl StableBytesMut for Good {
///     fn bytes_mut(&mut self) -> &mut [u8] {
///         self.data.as_mut()
///     }
/// }
/// ```
/// The above implementation is only safe if the `Vec<u8>` inside
/// of `Good` is only ever mutated through `StableBytesMut`.
/// Otherwise the bytes could be changed.
pub unsafe trait StableBytesMut: StableBytes {
    /// Gets the underlying bytes mutably.
    fn bytes_mut(&mut self) -> &mut [u8];
}

// ==============
// Implementations of `StableBytes` for popular types
// ==============

unsafe impl StableBytes for &[u8] {
    fn bytes(&self) -> &[u8] {
        self
    }
}

unsafe impl StableBytesMut for AlignedVec {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

unsafe impl StableBytes for AlignedVec {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytesMut for Vec<u8> {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

unsafe impl StableBytes for Vec<u8> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}



unsafe impl StableBytesMut for Box<[u8]> {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

unsafe impl StableBytes for Box<[u8]> {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use rkyv::{rancor, Archive, Deserialize, Serialize};

    use super::OwnedArchive;

    #[derive(Archive, Clone, PartialEq, Deserialize, Serialize, Debug)]
    #[rkyv(check_bytes, compare(PartialEq), derive(Debug))]
    pub struct ArchiveStub {
        hello: u8,
        world: u64,
    }

    #[test]
    fn test_owned_archive_vec() {
        let stub = ArchiveStub { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&stub).unwrap();
        let owned: OwnedArchive<ArchiveStub, _> =
            OwnedArchive::new::<rancor::Error>(bytes).unwrap();

        // Finally check to see that both are equal.
        assert_eq!(owned.hello, 4);
        assert_eq!(owned.world, 5);

        // Finally check to see that both are equal.
        assert_eq!(stub, *owned);
    }

    #[test]
    fn test_owned_archive_vec_mut() {
        let stub = ArchiveStub { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&stub).unwrap();
        let mut owned: OwnedArchive<ArchiveStub, _> =
            OwnedArchive::new::<rancor::Error>(bytes).unwrap();

        // Check that they are the same.
        assert_eq!(stub, *owned);

        owned.get_mut().hello = 4;

        assert_eq!(owned.hello, 4);
    }
}
