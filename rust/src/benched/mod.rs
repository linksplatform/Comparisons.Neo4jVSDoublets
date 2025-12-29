//! # Benchmark Lifecycle Management
//!
//! This module defines the [`Benched`] trait that provides setup/teardown lifecycle
//! for benchmark iterations. Both Neo4j and Doublets storage backends implement
//! this trait to enable fair benchmarking.
//!
//! ## Module Structure
//!
//! The implementations are split into separate files for clear comparison:
//!
//! - **[`doublets_benched`]** - Doublets storage backend implementations
//! - **[`neo4j_benched`]** - Neo4j storage backend implementations

mod doublets_benched;
mod neo4j_benched;

use crate::Fork;

/// Trait for types that can be benchmarked.
///
/// Provides the setup/teardown lifecycle for benchmark iterations:
/// - [`Benched::setup`] - Initialize the storage backend
/// - [`Benched::fork`] - Create an isolated environment for a single iteration
/// - [`Benched::unfork`] - Clean up after the iteration
pub trait Benched: Sized {
    /// Builder parameter type for constructing this storage.
    type Builder<'params>;

    /// Set up a new storage backend for benchmarking.
    fn setup<'a>(builder: Self::Builder<'a>) -> crate::Result<Self>;

    /// Create a fork for a single benchmark iteration.
    ///
    /// This allows each iteration to run in isolation without affecting others.
    fn fork(&mut self) -> Fork<Self> {
        Fork(self)
    }

    /// Clean up after a benchmark iteration.
    ///
    /// # Safety
    /// This method may perform unsafe operations like clearing all data.
    unsafe fn unfork(&mut self);
}
