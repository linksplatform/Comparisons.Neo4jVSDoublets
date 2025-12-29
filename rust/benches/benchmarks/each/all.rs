//! # Each All Benchmark
//!
//! Measures the performance of iterating over ALL links in the database.
//! This is equivalent to a full table scan with constraint `[*, *, *]`.
//!
//! ## Common Interface Method
//!
//! ```rust,ignore
//! store.each(|link| Flow::Continue);
//! // Equivalent to: store.each_by([any, any, any], handler)
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Iterates through internal link array sequentially
//! - Skips empty/deleted slots
//! - Time complexity: O(n) where n = total links
//!
//! ### Neo4j
//! ```cypher
//! MATCH (l:Link) RETURN l.id, l.source, l.target
//! ```

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::{
    data::{Flow, LinkType},
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_all benchmark on a specific storage backend.
fn bench<T: LinkType, B: Benched + Doublets<T>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    let handler = |_| Flow::Continue;
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            // The benchmarked operation: iterate all BACKGROUND_LINKS
            // This calls the same interface method on both Doublets and Neo4j
            let _ = elapsed! { fork.each(handler) };
        })(bencher, &mut benched);
    });
}

/// Creates benchmark comparing all storage backends on full table scan.
pub fn each_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_All");

    // =========================================================================
    // NEO4J BACKENDS
    // =========================================================================
    // Neo4j executes: MATCH (l:Link) RETURN l.id, l.source, l.target

    tri! {
        bench(&mut group, "Neo4j_NonTransaction", Exclusive::<Client<usize>>::setup(()).unwrap());
    }
    tri! {
        let client = connect().unwrap();
        bench(
            &mut group,
            "Neo4j_Transaction",
            Exclusive::<Transaction<'_, usize>>::setup(&client).unwrap(),
        );
    }

    // =========================================================================
    // DOUBLETS BACKENDS
    // =========================================================================
    // Doublets iterates internal array, skipping empty slots

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
