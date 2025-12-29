//! # Neo4j Each Identity Benchmark
//!
//! Measures the performance of looking up links by ID in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses unique constraint index on `id` property
//! - Time complexity: O(log n) + network overhead

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::data::{Flow, LinksConstants};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_identity benchmark on a Neo4j backend.
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
                elapsed! {fork.each_by([index, any, any], handler)};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Neo4j backends on ID lookup.
pub fn each_identity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Identity");

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
