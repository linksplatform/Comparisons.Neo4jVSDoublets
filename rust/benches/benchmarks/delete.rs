//! # Delete Links Benchmark
//!
//! This benchmark measures the performance of deleting links by ID in both
//! Doublets and Neo4j databases.
//!
//! ## Common Interface Method
//!
//! Both implementations use the `Doublets<T>::delete(id)` method:
//! ```rust,ignore
//! // Deletes a link by its ID
//! store.delete(id)?;
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Looks up link by ID (O(1) array access)
//! - Removes from source and target indexes
//! - Marks slot as free for reuse
//! - Time complexity: O(log n) for index updates
//!
//! ### Neo4j
//! Executes this Cypher query:
//! ```cypher
//! MATCH (l:Link {id: $id}) DELETE l
//! ```
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j finds node by indexed id property
//! - Removes node and updates indexes
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

/// Runs the delete benchmark on a specific storage backend.
///
/// The benchmark:
/// 1. Sets up a fresh database with BACKGROUND_LINKS existing links
/// 2. Creates LINK_COUNT additional links to delete
/// 3. Measures time to delete those LINK_COUNT links (in reverse order)
/// 4. Each call uses the same interface: `store.delete(id)`
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            // Setup: Create additional links beyond background links to delete
            for _prepare in BACKGROUND_LINKS..BACKGROUND_LINKS + *LINK_COUNT {
                let _ = fork.create_point();
            }
            // The benchmarked operation: delete LINK_COUNT links by ID
            // This calls the same interface method on both Doublets and Neo4j
            for id in (BACKGROUND_LINKS + 1..=BACKGROUND_LINKS + *LINK_COUNT).rev() {
                let _ = elapsed! {fork.delete(id)?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark comparing all storage backends on link deletion.
pub fn delete_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Delete");

    // =========================================================================
    // NEO4J BACKENDS
    // =========================================================================
    // Neo4j executes: MATCH (l:Link {id: $id}) DELETE l

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
    // Doublets removes link from storage and updates indexes

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
