//! # Neo4j Delete Links Benchmark
//!
//! This benchmark measures the performance of deleting links by ID in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link {id: $id}) DELETE l
//! ```
//!
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j finds node by indexed id property
//! - Removes node and updates indexes
//! - Time complexity: O(log n) + network overhead

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT};

use crate::tri;

/// Runs the delete benchmark on a Neo4j backend.
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

/// Creates benchmark for Neo4j backends on link deletion.
pub fn delete_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Delete");

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

    group.finish();
}
