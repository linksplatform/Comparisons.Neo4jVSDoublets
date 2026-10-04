//! # Neo4j Each Incoming Benchmark
//!
//! Runs [`crate::benchmarks::each_incoming`] on Neo4j in both transaction modes, and
//! the same work as one batch (see `batch.rs`).
//!
//! Index seek on `link_target`:
//! ```cypher
//! MATCH (l:Link) WHERE l.target = $target RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_incoming(c: &mut Criterion) {
    run(
        c,
        "Each_Incoming",
        crate::benchmarks::each_incoming,
        Some(super::super::batch::each_incoming),
    );
}
