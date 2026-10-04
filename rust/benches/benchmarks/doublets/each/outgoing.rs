//! # Doublets Each Outgoing Benchmark
//!
//! Runs [`crate::benchmarks::each_outgoing`] on all four Doublets stores.
//!
//! Traverses the source index tree.

use criterion::Criterion;

pub fn each_outgoing(c: &mut Criterion) {
    crate::run_doublets!(c, "Each_Outgoing", crate::benchmarks::each_outgoing);
}
