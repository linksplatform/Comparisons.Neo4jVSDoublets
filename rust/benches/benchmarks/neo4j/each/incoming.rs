//! # Neo4j Each Incoming Benchmark
//!
//! Runs [`crate::benchmarks::each_incoming`] on Neo4j in both transaction modes.
//!
//! Index seek on `link_target`:
//! ```cypher
//! MATCH (l:Link) WHERE l.target = $target RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_incoming(c: &mut Criterion) {
    run(c, "Each_Incoming", crate::benchmarks::each_incoming);
}
