//! # Doublets Delete Benchmark
//!
//! Runs [`crate::benchmarks::delete`] on all four Doublets stores.
//!
//! Detaches the link from the index trees and marks its index as free.

use criterion::Criterion;

pub fn delete_links(c: &mut Criterion) {
    crate::run_doublets!(c, "Delete", crate::benchmarks::delete);
}
