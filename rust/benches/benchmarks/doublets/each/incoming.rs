//! # Doublets Each Incoming Benchmark
//!
//! Runs [`crate::benchmarks::each_incoming`] on all four Doublets stores.
//!
//! Traverses the target index tree.

use criterion::Criterion;

pub fn each_incoming(c: &mut Criterion) {
    crate::run_doublets!(c, "Each_Incoming", crate::benchmarks::each_incoming);
}
