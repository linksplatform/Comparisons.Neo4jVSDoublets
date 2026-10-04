//! # Doublets Benched Implementations
//!
//! | Benchmark name                | Type                                        | Storage                  |
//! |-------------------------------|---------------------------------------------|--------------------------|
//! | `Doublets_United_Volatile`    | `unit::Store<T, Global<_>>`                 | RAM                      |
//! | `Doublets_United_NonVolatile` | `unit::Store<T, FileMapped<_>>`             | memory-mapped file       |
//! | `Doublets_Split_Volatile`     | `split::Store<T, Global<_>, Global<_>>`     | RAM                      |
//! | `Doublets_Split_NonVolatile`  | `split::Store<T, FileMapped<_>, FileMapped<_>>` | memory-mapped files  |
//!
//! The background links are created with `create_point()` and removed with
//! `delete_all()`.

use doublets::{
    data::LinkReference,
    mem::{FileMapped, Global},
    split::{self, DataPart, IndexPart},
    unit::{self, LinkPart},
    Doublets,
};

use super::Benched;
use crate::{map_file, Fork};

/// Implements the lifecycle that is the same for all Doublets stores.
macro_rules! doublets_lifecycle {
    () => {
        fn fork(&mut self, background_links: usize) -> crate::Result<Fork<'_, Self>> {
            for _ in 0..background_links {
                self.create_point()?;
            }
            Ok(Fork(self))
        }

        fn unfork(&mut self) -> crate::Result<()> {
            Ok(self.delete_all()?)
        }
    };
}

impl<T: LinkReference> Benched for unit::Store<T, Global<LinkPart<T>>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> crate::Result<Self> {
        Ok(Self::new(Global::new())?)
    }

    doublets_lifecycle!();
}

impl<T: LinkReference> Benched for unit::Store<T, FileMapped<LinkPart<T>>> {
    type Builder<'a> = &'a str;

    fn setup(path: Self::Builder<'_>) -> crate::Result<Self> {
        let mut store = Self::new(map_file(path)?)?;
        store.delete_all()?;
        Ok(store)
    }

    doublets_lifecycle!();
}

impl<T: LinkReference> Benched for split::Store<T, Global<DataPart<T>>, Global<IndexPart<T>>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> crate::Result<Self> {
        Ok(Self::new(Global::new(), Global::new())?)
    }

    doublets_lifecycle!();
}

impl<T: LinkReference> Benched
    for split::Store<T, FileMapped<DataPart<T>>, FileMapped<IndexPart<T>>>
{
    type Builder<'a> = (&'a str, &'a str);

    fn setup((data, index): Self::Builder<'_>) -> crate::Result<Self> {
        let mut store = Self::new(map_file(data)?, map_file(index)?)?;
        store.delete_all()?;
        Ok(store)
    }

    doublets_lifecycle!();
}
