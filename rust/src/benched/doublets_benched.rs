//! # Doublets Benched Implementations
//!
//! This module contains the [`Benched`] trait implementations for all Doublets
//! storage backends.
//!
//! ## Storage Backends
//!
//! | Type                         | Storage        | Description                      |
//! |------------------------------|----------------|----------------------------------|
//! | `unit::Store` + `FileMapped` | Non-volatile   | Memory-mapped file storage       |
//! | `unit::Store` + `Alloc`      | Volatile       | In-memory storage                |
//! | `split::Store` + `FileMapped`| Non-volatile   | Split data/index file storage    |
//! | `split::Store` + `Alloc`     | Volatile       | Split data/index in-memory       |
//!
//! ## Implementation Details
//!
//! All Doublets implementations clean up by calling `delete_all()` in `unfork()`,
//! which removes all links from the storage for the next benchmark iteration.

use std::alloc::Global;

use doublets::{
    data::LinkType,
    mem::{Alloc, FileMapped},
    split::{self, DataPart, IndexPart},
    unit::{self, LinkPart},
    Doublets,
};

use super::Benched;
use crate::map_file;

/// Doublets United (unit) store with non-volatile (file-mapped) storage.
///
/// ## Setup
/// ```rust,ignore
/// let store = unit::Store::<usize, FileMapped<LinkPart<_>>>::setup("united.links")?;
/// ```
///
/// ## Cleanup
/// Calls `delete_all()` to remove all links between iterations.
impl<T: LinkType> Benched for unit::Store<T, FileMapped<LinkPart<T>>> {
    type Builder<'a> = &'a str;

    fn setup(builder: Self::Builder<'_>) -> crate::Result<Self> {
        Self::new(map_file(builder)?).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

/// Doublets United (unit) store with volatile (in-memory) storage.
///
/// ## Setup
/// ```rust,ignore
/// let store = unit::Store::<usize, Alloc<LinkPart<_>, Global>>::setup(())?;
/// ```
///
/// ## Cleanup
/// Calls `delete_all()` to remove all links between iterations.
impl<T: LinkType> Benched for unit::Store<T, Alloc<LinkPart<T>, Global>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> crate::Result<Self> {
        Self::new(Alloc::new(Global)).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

/// Doublets Split store with non-volatile (file-mapped) storage.
///
/// ## Setup
/// ```rust,ignore
/// let store = split::Store::<usize, FileMapped<_>, FileMapped<_>>::setup(
///     ("split_index.links", "split_data.links")
/// )?;
/// ```
///
/// ## Cleanup
/// Calls `delete_all()` to remove all links between iterations.
impl<T: LinkType> Benched for split::Store<T, FileMapped<DataPart<T>>, FileMapped<IndexPart<T>>> {
    type Builder<'a> = (&'a str, &'a str);

    fn setup((data, index): Self::Builder<'_>) -> crate::Result<Self> {
        Self::new(map_file(data)?, map_file(index)?).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}

/// Doublets Split store with volatile (in-memory) storage.
///
/// ## Setup
/// ```rust,ignore
/// let store = split::Store::<usize, Alloc<DataPart<_>, _>, Alloc<IndexPart<_>, _>>::setup(())?;
/// ```
///
/// ## Cleanup
/// Calls `delete_all()` to remove all links between iterations.
impl<T: LinkType> Benched
    for split::Store<T, Alloc<DataPart<T>, Global>, Alloc<IndexPart<T>, Global>>
{
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> crate::Result<Self> {
        Self::new(Alloc::new(Global), Alloc::new(Global)).map_err(Into::into)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.delete_all();
    }
}
