//! # Doublets Benchmarks
//!
//! Every operation runs on four stores:
//!
//! | Benchmark name                | Store                                    |
//! |-------------------------------|------------------------------------------|
//! | `Doublets_United_Volatile`    | unit store in RAM                        |
//! | `Doublets_United_NonVolatile` | unit store in a memory-mapped file       |
//! | `Doublets_Split_Volatile`     | split store in RAM                       |
//! | `Doublets_Split_NonVolatile`  | split store in memory-mapped files       |
//!
//! See [`linksneo4j::doublets_impl`] for how every operation is implemented.

mod create;
mod delete;
pub mod each;
mod update;

pub use create::create_links;
pub use delete::delete_links;
pub use each::*;
pub use update::update_links;

/// Runs `$operation` on all four Doublets stores.
///
/// A macro is used because every store is a different type.
#[macro_export]
macro_rules! run_doublets {
    ($c:expr, $group_name:literal, $operation:path) => {{
        use linksneo4j::{
            Benched, DoubletsSplitNonVolatile, DoubletsSplitVolatile, DoubletsUnitedNonVolatile,
            DoubletsUnitedVolatile,
        };

        let mut group = $crate::benchmarks::doublets_group($c, $group_name);
        $operation(
            &mut group,
            "Doublets_United_Volatile",
            &mut DoubletsUnitedVolatile::setup(()).unwrap(),
        );
        $operation(
            &mut group,
            "Doublets_United_NonVolatile",
            &mut DoubletsUnitedNonVolatile::setup("united.links").unwrap(),
        );
        $operation(
            &mut group,
            "Doublets_Split_Volatile",
            &mut DoubletsSplitVolatile::setup(()).unwrap(),
        );
        $operation(
            &mut group,
            "Doublets_Split_NonVolatile",
            &mut DoubletsSplitNonVolatile::setup(("split_data.links", "split_index.links"))
                .unwrap(),
        );
        group.finish();
    }};
}
