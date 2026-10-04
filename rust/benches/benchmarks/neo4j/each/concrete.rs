//! # Neo4j Each Concrete Benchmark
//!
//! Runs [`crate::benchmarks::each_concrete`] on Neo4j in both transaction modes, and
//! the same work as one batch (see `batch.rs`).
//!
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source AND l.target = $target
//! RETURN l.id AS id, l.source AS source, l.target AS target
//! ```

use criterion::Criterion;

use super::super::run;

pub fn each_concrete(c: &mut Criterion) {
    run(
        c,
        "Each_Concrete",
        crate::benchmarks::each_concrete,
        Some(super::super::batch::each_concrete),
    );
}
