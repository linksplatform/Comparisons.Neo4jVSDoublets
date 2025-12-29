//! # Doublets Delete Links Benchmark
//!
//! This benchmark measures the performance of deleting links by ID in Doublets.
//!
//! ## Implementation
//!
//! Doublets deletes links by:
//! - Looking up link by ID (O(1) array access)
//! - Removing from source and target indexes
//! - Marking slot as free for reuse
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

/// Runs the delete benchmark on a Doublets backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            for _prepare in BACKGROUND_LINKS..BACKGROUND_LINKS + *LINK_COUNT {
                let _ = fork.create_point();
            }
            for id in (BACKGROUND_LINKS + 1..=BACKGROUND_LINKS + *LINK_COUNT).rev() {
                let _ = elapsed! {fork.delete(id)?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Doublets backends on link deletion.
pub fn delete_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Delete");

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
