//! # Neo4j Create Links Benchmark
//!
//! This benchmark measures the performance of creating new links in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! CREATE (l:Link {id: $id, source: 0, target: 0})
//! ```
//!
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j allocates node, sets properties
//! - Updates indexes on id, source, target
//! - Time complexity: O(log n) + network overhead

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT};

use crate::tri;

/// Runs the create benchmark on a Neo4j backend.
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

/// Creates benchmark for Neo4j backends on link creation.
pub fn create_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create");

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
