//! Various utilities for use with [rkyv].
//!
//! [rkyv]: https://github.com/rkyv/rkyv

#![deny(
    rustdoc::broken_intra_doc_links,
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unsafe_op_in_unsafe_fn
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(docsrs, not(doctest)), feature(doc_cfg, doc_auto_cfg))]
#![doc(html_favicon_url = r#"
    data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0
    26.458 26.458'%3E%3Cpath d='M0 0v26.458h26.458V0zm9.175 3.772l8.107 8.106
    2.702-2.702 2.702 13.512-13.512-2.702 2.703-2.702-8.107-8.107z'/%3E
    %3C/svg%3E
"#)]
#![doc(html_logo_url = r#"
    data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="100"
    height="100" viewBox="0 0 26.458 26.458"%3E%3Cpath d="M0
    0v26.458h26.458V0zm9.175 3.772l8.107 8.106 2.702-2.702 2.702
    13.512-13.512-2.702 2.703-2.702-8.107-8.107z"/%3E%3C/svg%3E
"#)]

pub mod owned;

#[cfg(feature = "memmap2")]
pub mod mmap;

#[cfg(feature = "std")]
pub mod std;
