//! # Neo4j Each Concrete Benchmark
//!
//! Measures the performance of querying links by BOTH source AND target in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $s AND l.target = $t
//! RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses one of the indexes (source or target), then filters
//! - Returns links matching both constraints

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::data::{Flow, LinksConstants};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_concrete benchmark on a Neo4j backend.
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

/// Creates benchmark for Neo4j backends on composite index lookup.
pub fn each_concrete(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Concrete");

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
