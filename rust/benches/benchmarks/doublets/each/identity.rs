//! # Doublets Each Identity Benchmark
//!
//! Runs [`crate::benchmarks::each_identity`] on all four Doublets stores.
//!
//! Reads `links[id]` directly.

use criterion::Criterion;

pub fn each_identity(c: &mut Criterion) {
    crate::run_doublets!(c, "Each_Identity", crate::benchmarks::each_identity);
}
