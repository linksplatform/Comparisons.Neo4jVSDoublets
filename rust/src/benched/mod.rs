//! # Benchmark Lifecycle
//!
//! This module defines the [`Benched`] trait, which prepares a store for a
//! benchmark. Both Neo4j and Doublets implement it:
//!
//! - **[`doublets_benched`]** - Doublets stores
//! - **[`neo4j_benched`]** - Neo4j

mod doublets_benched;
mod neo4j_benched;

use crate::Fork;

/// A store that can be benchmarked.
///
/// A benchmark of one operation is:
/// 1. [`Benched::fork`] - not measured: create the background links
/// 2. for every iteration:
///    - [`Benched::begin`], the operation, [`Benched::commit`] - measured
///    - undo the changes of the operation - not measured
/// 3. [`Benched::unfork`] - not measured: remove all links
pub trait Benched: Sized {
    /// Parameters needed to construct this store.
    type Builder<'params>;

    /// Opens the store and removes all links from it.
    fn setup(builder: Self::Builder<'_>) -> crate::Result<Self>;

    /// Creates the point links `1..=background_links` (each with
    /// `id = source = target`) in the empty store.
    ///
    /// The returned [`Fork`] removes all links again when it is dropped.
    fn fork(&mut self, background_links: usize) -> crate::Result<Fork<'_, Self>>;

    /// Starts a transaction. Only Neo4j in transaction mode has one.
    fn begin(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Commits the transaction started by [`Benched::begin`]. For all other
    /// stores every operation is already complete when it returns.
    fn commit(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Removes all links.
    fn unfork(&mut self) -> crate::Result<()>;
}
