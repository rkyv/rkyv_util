//! Enables us to pass around owned archive types.



pub struct OwnedArchive<T, C: StableBytes> {
    
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
///
/// ```
///
pub unsafe trait StableBytes {
    /// Gets the underlying bytes.
    fn bytes(&self) -> &[u8];
}


#[cfg(test)]
mod tests {

    
}
