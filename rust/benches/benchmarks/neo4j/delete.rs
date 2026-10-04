//! # Neo4j Delete Benchmark
//!
//! Runs [`crate::benchmarks::delete`] on Neo4j in both transaction modes, and
//! the same work as one batch (see `batch.rs`).
//!
//! One statement per deleted link, which also returns the previous values:
//! ```cypher
//! MATCH (l:Link {id: $id})
//! WITH l, l.source AS source, l.target AS target
//! DELETE l
//! RETURN source, target
//! ```

use criterion::Criterion;

use super::run;

pub fn delete_links(c: &mut Criterion) {
    run(
        c,
        "Delete",
        crate::benchmarks::delete,
        Some(super::batch::delete),
    );
}
