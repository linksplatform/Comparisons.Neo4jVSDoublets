//! # Create Links Benchmark
//!
//! This benchmark measures the performance of creating new links (nodes) in both
//! Doublets and Neo4j databases.
//!
//! ## Common Interface Method
//!
//! Both implementations use the `Doublets<T>::create_point()` method:
//! ```rust,ignore
//! // Creates a "point" link where id = source = target
//! store.create_point()?;
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Allocates next available ID from internal counter
//! - Writes (id, id, id) tuple directly to memory/file
//! - Updates source and target indexes
//! - Time complexity: O(log n) for index updates
//!
//! ### Neo4j
//! Executes this Cypher query:
//! ```cypher
//! CREATE (l:Link {id: $id, source: 0, target: 0})
//! ```
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j allocates node, sets properties
//! - Updates indexes on id, source, target
//! - Time complexity: O(log n) + network overhead

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::{
    data::LinkType,
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT};

use crate::tri;

/// Runs the create benchmark on a specific storage backend.
///
/// The benchmark:
/// 1. Sets up a fresh database with BACKGROUND_LINKS existing links
/// 2. Measures time to create LINK_COUNT new point links
/// 3. Each call uses the same interface: `store.create_point()`
fn bench<T: LinkType, B: Benched + Doublets<T>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            // The benchmarked operation: create LINK_COUNT point links
            // This calls the same interface method on both Doublets and Neo4j
            for _ in 0..*LINK_COUNT {
                let _ = elapsed! {fork.create_point()?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark comparing all storage backends on link creation.
pub fn create_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create");

    // =========================================================================
    // NEO4J BACKENDS
    // =========================================================================
    // Neo4j executes: CREATE (l:Link {id: $id, source: 0, target: 0})

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
    // Doublets writes (id, id, id) to storage and updates indexes

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
