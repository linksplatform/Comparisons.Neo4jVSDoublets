//! # Doublets Each Concrete Benchmark
//!
//! Runs [`crate::benchmarks::each_concrete`] on all four Doublets stores.
//!
//! Searches the source index tree for the `(source, target)` pair.

use criterion::Criterion;

pub fn each_concrete(c: &mut Criterion) {
    crate::run_doublets!(c, "Each_Concrete", crate::benchmarks::each_concrete);
}
