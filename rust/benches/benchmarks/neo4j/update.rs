//! # Neo4j Update Benchmark
//!
//! Runs [`crate::benchmarks::update`] on Neo4j in both transaction modes.
//!
//! One statement per update, which also returns the previous values:
//! ```cypher
//! MATCH (l:Link {id: $id})
//! WITH l, l.source AS source, l.target AS target
//! SET l.source = $new_source, l.target = $new_target
//! RETURN source, target
//! ```

use criterion::Criterion;

use super::run;

pub fn update_links(c: &mut Criterion) {
    run(c, "Update", crate::benchmarks::update);
}
