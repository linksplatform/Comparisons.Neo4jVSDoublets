//! # Neo4j Each Outgoing Benchmark
//!
//! Measures the performance of querying links by source (outgoing edges) in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source
//! RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses index on `source`
//! - Returns all outgoing edges from a node

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::data::{Flow, LinksConstants};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_outgoing benchmark on a Neo4j backend.
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
                let _ = elapsed! {fork.each_by([any, index, any], handler)};
            }
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Neo4j backends on source index lookup.
pub fn each_outgoing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Outgoing");

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
