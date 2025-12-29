//! # Doublets Each Concrete Benchmark
//!
//! Measures the performance of querying links by BOTH source AND target in Doublets.
//!
//! ## Implementation
//!
//! Doublets queries by source AND target using:
//! - Uses source OR target index tree to find candidates
//! - Filters by the other field
//! - Time complexity: O(log n) for tree traversal

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::data::{Flow, LinksConstants};
use doublets::{
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, Benched, Fork};

use crate::tri;

/// Runs the each_concrete benchmark on a Doublets backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    let handler = |_| Flow::Continue;
    let any = LinksConstants::new().any;
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            for index in 1..=BACKGROUND_LINKS {
                elapsed! {fork.each_by([any, index, index], handler)};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Doublets backends on composite index lookup.
pub fn each_concrete(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Concrete");

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
