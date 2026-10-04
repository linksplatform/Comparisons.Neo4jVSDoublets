//! # Neo4j Each Outgoing Benchmark
//!
//! Runs [`crate::benchmarks::each_outgoing`] on Neo4j in both transaction modes.
//!
//! Index seek on `link_source`:
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_outgoing(c: &mut Criterion) {
    run(c, "Each_Outgoing", crate::benchmarks::each_outgoing);
}
