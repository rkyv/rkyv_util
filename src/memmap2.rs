//! Bindings for working with memory mapped objects in a cleaner way.

use memmap2_0_9::{Mmap, MmapMut};

use crate::owned::{StableBytes, StableBytesMut};

unsafe impl StableBytes for Mmap {
    fn bytes(&self) -> &[u8] {
        &*self
    }
}

unsafe impl StableBytes for MmapMut {
    fn bytes(&self) -> &[u8] {
        &*self
    }
}

unsafe impl StableBytesMut for MmapMut {
    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut *self
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Write};
    use memmap2_0_9::{Mmap, MmapMut};
    use rkyv::{rancor, Archive, Deserialize, Serialize, munge::munge};

    use crate::owned::OwnedArchive;

    #[derive(Archive, Clone, PartialEq, Deserialize, Serialize, Debug)]
    #[rkyv(compare(PartialEq), derive(Debug))]
    pub struct Foo {
        hello: u8,
        world: u64,
    }

    #[test]
    fn test_owned_archive_vec_mmap() {
        let foo = Foo { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&foo).unwrap();

        let mut tfile = tempfile::tempfile().unwrap();

        tfile.write_all(&bytes).unwrap();
        tfile.seek(SeekFrom::Start(0)).unwrap();

        let mmap = unsafe { Mmap::map(&tfile) }.unwrap();

        let owned = OwnedArchive::<Foo, _>::new::<rancor::Error>(mmap).unwrap();

        // Finally check to see that both are equal.
        assert_eq!(owned.hello, 4);
        assert_eq!(owned.world, 5);

        // Finally check to see that both are equal.
        assert_eq!(foo, *owned);
    }

    #[test]
    fn test_owned_archive_vec_mmap_mut() {
        let foo = Foo { hello: 4, world: 5 };

        let bytes = rkyv::to_bytes::<rancor::Error>(&foo).unwrap();

        let mut tfile = tempfile::tempfile().unwrap();

        tfile.write_all(&bytes).unwrap();
        tfile.seek(SeekFrom::Start(0)).unwrap();
        // write(tfile.path(), contents)

        let mmap = unsafe { MmapMut::map_mut(&tfile) }.unwrap();

        let mut owned = OwnedArchive::<Foo, _>::new::<rancor::Error>(mmap).unwrap();

        // Finally check to see that both are equal.
        assert_eq!(owned.hello, 4);
        assert_eq!(owned.world, 5);

        // Finally check to see that both are equal.
        assert_eq!(foo, *owned);

        // Modify it
        munge!(let ArchivedFoo { mut hello, .. } = owned.get_mut());
        *hello = 3;
        assert_eq!(owned.hello, 3);
    }
}
