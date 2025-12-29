//! # Update Links Benchmark
//!
//! This benchmark measures the performance of updating existing links in both
//! Doublets and Neo4j databases.
//!
//! ## Common Interface Method
//!
//! Both implementations use the `Doublets<T>::update(id, source, target)` method:
//! ```rust,ignore
//! // Updates a link's source and target by its ID
//! store.update(id, new_source, new_target)?;
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Looks up link by ID (O(1) array access)
//! - Updates source/target values in storage
//! - Re-indexes in source and target trees if values changed
//! - Time complexity: O(log n) for index updates
//!
//! ### Neo4j
//! Executes this Cypher query:
//! ```cypher
//! MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target
//! ```
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j finds node by indexed id property
//! - Updates properties and re-indexes
//! - Time complexity: O(log n) + network overhead

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
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT};

use crate::tri;

/// Runs the update benchmark on a specific storage backend.
///
/// The benchmark:
/// 1. Sets up a fresh database with BACKGROUND_LINKS existing links
/// 2. Measures time to update LINK_COUNT links (twice each: to 0,0 then to id,id)
/// 3. Each call uses the same interface: `store.update(id, source, target)`
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            // Update the last LINK_COUNT links from background links
            let start_id = if BACKGROUND_LINKS > *LINK_COUNT { BACKGROUND_LINKS - *LINK_COUNT + 1 } else { 1 };
            for id in start_id..=BACKGROUND_LINKS {
                // The benchmarked operation: update links twice
                // This calls the same interface method on both Doublets and Neo4j
                let _ = elapsed! {fork.update(id, 0, 0)?};  // Reset to (0, 0)
                let _ = elapsed! {fork.update(id, id, id)?}; // Set to point link (id, id)
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark comparing all storage backends on link updates.
pub fn update_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Update");

    // =========================================================================
    // NEO4J BACKENDS
    // =========================================================================
    // Neo4j executes: MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target

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
    // Doublets updates (source, target) in storage and re-indexes

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
