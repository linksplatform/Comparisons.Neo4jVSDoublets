//! # Neo4j Create Benchmark
//!
//! Runs [`crate::benchmarks::create`] on Neo4j in both transaction modes.
//!
//! One statement per created link:
//! ```cypher
//! CREATE (:Link {id: $id, source: $id, target: $id})
//! ```

use criterion::Criterion;

use super::run;

pub fn create_links(c: &mut Criterion) {
    run(c, "Create", crate::benchmarks::create);
}
