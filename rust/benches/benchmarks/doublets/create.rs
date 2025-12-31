//! # Doublets Create Links Benchmark
//!
//! This benchmark measures the performance of creating new links in Doublets.
//!
//! ## Implementation
//!
//! Doublets creates links by:
//! - Allocating next available ID from internal counter
//! - Writing (id, id, id) tuple directly to memory/file
//! - Updating source and target indexes
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

/// Runs the create benchmark on a Doublets backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            for _ in 0..*LINK_COUNT {
                let _ = elapsed! {fork.create_point()?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Doublets backends on link creation.
pub fn create_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create");

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
