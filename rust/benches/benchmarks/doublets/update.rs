//! # Doublets Update Links Benchmark
//!
//! This benchmark measures the performance of updating existing links in Doublets.
//!
//! ## Implementation
//!
//! Doublets updates links by:
//! - Looking up link by ID (O(1) array access)
//! - Updating source/target values in storage
//! - Re-indexing in source and target trees if values changed
//! - Time complexity: O(log n) for index updates

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::{
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, Benched, Fork, LINK_COUNT};

use crate::tri;

/// Runs the update benchmark on a Doublets backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            let start_id = if BACKGROUND_LINKS > *LINK_COUNT { BACKGROUND_LINKS - *LINK_COUNT + 1 } else { 1 };
            for id in start_id..=BACKGROUND_LINKS {
                let _ = elapsed! {fork.update(id, 0, 0)?};
                let _ = elapsed! {fork.update(id, id, id)?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Doublets backends on link updates.
pub fn update_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Update");

    tri! {
        bench(
            &mut group,
            "Doublets_United_Volatile",
            unit::Store::<usize, Alloc<LinkPart<_>, Global>>::setup(()).unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_United_NonVolatile",
            unit::Store::<usize, FileMapped<LinkPart<_>>>::setup("united.links").unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_Split_Volatile",
            split::Store::<usize, Alloc<DataPart<_>, _>, Alloc<IndexPart<_>, _>>::setup(()).unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_Split_NonVolatile",
            split::Store::<usize, FileMapped<_>, FileMapped<_>>::setup(("split_index.links", "split_data.links")).unwrap()
        )
    }

    group.finish();
}
