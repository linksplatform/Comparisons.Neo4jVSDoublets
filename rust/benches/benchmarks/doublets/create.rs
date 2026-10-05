//! # Doublets Create Benchmark
//!
//! Runs [`crate::benchmarks::create`] on all four Doublets stores.
//!
//! Takes the next free index, writes `(id, id, id)` and inserts the link into
//! the source and target index trees.

use criterion::Criterion;

pub fn create_links(c: &mut Criterion) {
    crate::run_doublets!(c, "Create", crate::benchmarks::create);
}
