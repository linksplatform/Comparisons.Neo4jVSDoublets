//! # Neo4j Update Links Benchmark
//!
//! This benchmark measures the performance of updating existing links in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target
//! ```
//!
//! - Makes HTTP request to `/db/neo4j/tx/commit`
//! - Neo4j finds node by indexed id property
//! - Updates properties and re-indexes
//! - Time complexity: O(log n) + network overhead

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT};

use crate::tri;

/// Runs the update benchmark on a Neo4j backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            let start_id = if BACKGROUND_LINKS > *LINK_COUNT { BACKGROUND_LINKS - *LINK_COUNT + 1 } else { 1 };
            for id in start_id..=BACKGROUND_LINKS {
                let _ = elapsed! {fork.update(id, 0, 0)?};
                let _ = elapsed! {fork.update(id, id, id)?};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Neo4j backends on link updates.
pub fn update_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Update");

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
