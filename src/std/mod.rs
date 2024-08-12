//! Provides implementations for certain objects in the standard library.
//! This facilitates the use of no-std.

use std::{rc::Rc, sync::Arc};

use crate::owned::StableBytes;

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
