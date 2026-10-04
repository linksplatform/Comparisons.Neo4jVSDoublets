//! # Neo4j Each All Benchmark
//!
//! Runs [`crate::benchmarks::each_all`] on Neo4j in both transaction modes.
//!
//! ```cypher
//! MATCH (l:Link) RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_all(c: &mut Criterion) {
    run(c, "Each_All", crate::benchmarks::each_all, None);
}
