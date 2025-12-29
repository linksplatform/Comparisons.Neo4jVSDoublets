//! # Each Identity Benchmark
//!
//! Measures the performance of looking up links by their ID (primary key).
//! This is equivalent to a primary key lookup with constraint `[id, *, *]`.
//!
//! ## Common Interface Method
//!
//! ```rust,ignore
//! // Query by ID (identity constraint)
//! store.each_by([id, any, any], |link| Flow::Continue);
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Direct array index access: O(1)
//! - Returns link at `links[id]` if it exists
//!
//! ### Neo4j
//! ```cypher
//! MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target
//! ```
//! - Uses unique constraint index on `id` property
//! - Time complexity: O(log n) + network overhead

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::{
    data::{Flow, LinksConstants},
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_identity benchmark on a specific storage backend.
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
            // The benchmarked operation: query each link by its ID
            // This calls the same interface method on both Doublets and Neo4j
            for index in 1..=BACKGROUND_LINKS {
                elapsed! {fork.each_by([index, any, any], handler)};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark comparing all storage backends on ID lookup.
pub fn each_identity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Identity");

    // =========================================================================
    // NEO4J BACKENDS
    // =========================================================================
    // Neo4j executes: MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target

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
    // Doublets uses direct array indexing: links[id]

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
