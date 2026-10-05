//! # Neo4j Each Identity Benchmark
//!
//! Runs [`crate::benchmarks::each_identity`] on Neo4j in both transaction modes, and
//! the same work as one batch (see `batch.rs`).
//!
//! Index seek on the `link_id` uniqueness constraint:
//! ```cypher
//! MATCH (l:Link) WHERE l.id = $id RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_identity(c: &mut Criterion) {
    run(
        c,
        "Each_Identity",
        crate::benchmarks::each_identity,
        Some(super::super::batch::each_identity),
    );
}
