//! # Doublets Each All Benchmark
//!
//! Runs [`crate::benchmarks::each_all`] on all four Doublets stores.
//!
//! Iterates over the links array, skipping free indexes.

use criterion::Criterion;

pub fn each_all(c: &mut Criterion) {
    crate::run_doublets!(c, "Each_All", crate::benchmarks::each_all);
}
