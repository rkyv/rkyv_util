//! Bindings for working with memory mapped objects in a cleaner way.

use std::ops::{Deref, DerefMut};

use memmap2::{Mmap, MmapMut};
use rkyv::{
    api::high::HighValidator, bytecheck::CheckBytes, rancor::Source, Archive,
    Portable,
};

use crate::owned::{OwnedArchive, StableBytes, StableBytesMut};

impl<T> OwnedArchive<T, ContractMmap> {
    /// Creates an OwnedArchive from a memory mapped object.
    ///
    /// You cannot use the `new` method to construct an OwnedArchive
    /// because the [StableBytes] and [StableBytesMut] interfaces only
    /// apply to a newtype wrapper. This is to make sure people do not
    /// casually create these types without taking into consideration the
    /// relevant safety invariants.
    ///
    /// # Safety
    /// This should hold up the same invariants as the [memmap2] crate.
    /// More specifically, if the underlying file is modified the buffer could
    /// change, therefore compromising the stability. One should therefore
    /// guarantee that the underlying file holds up the safety invariants
    /// set forth by [StableBytes].
    pub unsafe fn from_mmap<E>(container: Mmap) -> Result<Self, E>
    where
        T: Archive,
        T::Archived: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
        E: Source,
    {
        Self::new(ContractMmap(container))
    }
}

impl<T> OwnedArchive<T, ContractMmapMut> {
    /// Creates an OwnedArchive from a mutable memory mapped object.
    ///
    /// You cannot use the `new` method to construct an OwnedArchive
    /// because the [StableBytes] and [StableBytesMut] interfaces only
    /// apply to a newtype wrapper. This is to make sure people do not
    /// casually create these types without taking into consideration the
    /// relevant safety invariants.
    ///
    /// # Safety
    /// This should hold up the same invariants as the [memmap2] crate.
    /// More specifically, if the underlying file is modified the buffer could
    /// change, therefore compromising the stability. One should therefore
    /// guarantee that the underlying file holds up the safety invariants
    /// set forth by [StableBytes].
    pub unsafe fn from_mmap_mut<E>(container: MmapMut) -> Result<Self, E>
    where
        T: Archive,
        T::Archived: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
        E: Source,
    {
        Self::new(ContractMmapMut(container))
    }
}

/// A newtype wrapper around the [Mmap] type. This prevents the creation of
/// [OwnedArchive] through the `new` method and therefore causes the programmer
/// to think about the relevant safety invariants that must be held up.
struct ContractMmap(Mmap);

impl Deref for ContractMmap {
    type Target = Mmap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A newtype wrapper around the [MmapMut] type. This prevents the creation of
/// [OwnedArchive] through the `new` method and therefore causes the programmer
/// to think about the relevant safety invariants that must be held up.
struct ContractMmapMut(MmapMut);

impl Deref for ContractMmapMut {
    type Target = MmapMut;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContractMmapMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl StableBytes for ContractMmap {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytes for ContractMmapMut {
    fn bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl StableBytesMut for ContractMmapMut {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

#[cfg(test)]
mod tests {

    use ::std::io::{Seek, SeekFrom, Write};
    use memmap2::{Mmap, MmapMut};
    use rkyv::{rancor, Archive, Deserialize, Serialize};

    use super::OwnedArchive;

    #[derive(Archive, Clone, PartialEq, Deserialize, Serialize, Debug)]
    #[rkyv(check_bytes, compare(PartialEq), derive(Debug))]
    pub struct ArchiveStub {
        hello: u8,
        world: u64,
    }

    #[test]
    fn test_owned_archive_vec_mmap() {
        let stub = ArchiveStub { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&stub).unwrap();

        let mut tfile = tempfile::tempfile().unwrap();

        tfile.write_all(&bytes).unwrap();
        tfile.seek(SeekFrom::Start(0)).unwrap();
        // write(tfile.path(), contents)

        let mmap = unsafe { Mmap::map(&tfile) }.unwrap();

        let owned: OwnedArchive<ArchiveStub, _> =
            unsafe { OwnedArchive::from_mmap::<rancor::Error>(mmap) }.unwrap();

        // Finally check to see that both are equal.
        assert_eq!(owned.hello, 4);
        assert_eq!(owned.world, 5);

        // Finally check to see that both are equal.
        assert_eq!(stub, *owned);
    }

    #[test]
    fn test_owned_archive_vec_mmap_mut() {
        let stub = ArchiveStub { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&stub).unwrap();

        let mut tfile = tempfile::tempfile().unwrap();

        tfile.write_all(&bytes).unwrap();
        tfile.seek(SeekFrom::Start(0)).unwrap();
        // write(tfile.path(), contents)

        let mmap = unsafe { MmapMut::map_mut(&tfile) }.unwrap();

        let mut owned: OwnedArchive<ArchiveStub, _> =
            unsafe { OwnedArchive::from_mmap_mut::<rancor::Error>(mmap) }
                .unwrap();

        // Finally check to see that both are equal.
        assert_eq!(owned.hello, 4);
        assert_eq!(owned.world, 5);

        // Finally check to see that both are equal.
        assert_eq!(stub, *owned);

        // Modify it
        owned.get_mut().hello = 3;
        assert_eq!(owned.hello, 3);
    }
}
