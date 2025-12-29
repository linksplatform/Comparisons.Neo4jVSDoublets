//! # Neo4j Each All Benchmark
//!
//! Measures the performance of iterating over ALL links in Neo4j.
//!
//! ## Implementation
//!
//! Neo4j executes this Cypher query:
//! ```cypher
//! MATCH (l:Link) RETURN l.id, l.source, l.target
//! ```
//!
//! - Full table scan
//! - Returns all nodes with :Link label

use std::time::{Duration, Instant};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::data::{Flow, LinkType};
use doublets::Doublets;
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_all benchmark on a Neo4j backend.
fn bench<T: LinkType, B: Benched + Doublets<T>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    let handler = |_| Flow::Continue;
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            let _ = elapsed! { fork.each(handler) };
        })(bencher, &mut benched);
    });
}

/// Creates benchmark for Neo4j backends on full table scan.
pub fn each_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_All");

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
