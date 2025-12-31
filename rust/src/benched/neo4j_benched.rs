//! # Neo4j Benched Implementations
//!
//! This module contains the [`Benched`] trait implementations for all Neo4j
//! storage backends.
//!
//! ## Storage Backends
//!
//! | Type                        | Mode            | Description                      |
//! |-----------------------------|-----------------|----------------------------------|
//! | `Exclusive<Client>`         | Non-transaction | Direct HTTP API calls            |
//! | `Exclusive<Transaction>`    | Transaction     | Transaction wrapper (same impl)  |
//!
//! ## Implementation Details
//!
//! Neo4j implementations clean up by executing:
//! ```cypher
//! MATCH (l:Link) DETACH DELETE l
//! ```
//!
//! This removes all Link nodes from the database for the next benchmark iteration.

use doublets::data::LinkType;

use super::Benched;
use crate::{Client, Exclusive, Fork, Sql, Transaction};

/// Neo4j client (non-transactional mode).
///
/// ## Setup
/// ```rust,ignore
/// let client = Exclusive::<Client<usize>>::setup(())?;
/// ```
///
/// ## Fork Behavior
/// Creates the schema (constraints/indexes) before each iteration.
///
/// ## Cleanup
/// Executes `MATCH (l:Link) DETACH DELETE l` to remove all nodes.
impl<T: LinkType> Benched for Exclusive<Client<T>> {
    type Builder<'a> = ();

    fn setup(_: Self::Builder<'_>) -> crate::Result<Self> {
        unsafe { Ok(Exclusive::new(crate::connect()?)) }
    }

    fn fork(&mut self) -> Fork<Self> {
        let _ = self.create_table();
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.drop_table();
    }
}

/// Neo4j transaction wrapper.
///
/// ## Setup
/// ```rust,ignore
/// let client = connect()?;
/// let transaction = Exclusive::<Transaction<'_, usize>>::setup(&client)?;
/// ```
///
/// ## Fork Behavior
/// Cleans up any existing data before each iteration to ensure isolation.
///
/// ## Cleanup
/// Executes `MATCH (l:Link) DETACH DELETE l` to remove all nodes.
impl<'a, T: LinkType> Benched for Exclusive<Transaction<'a, T>> {
    type Builder<'b> = &'a Client<T>;

    fn setup(builder: Self::Builder<'_>) -> crate::Result<Self> {
        let transaction = Transaction::new(builder)?;
        unsafe { Ok(Exclusive::new(transaction)) }
    }

    fn fork(&mut self) -> Fork<Self> {
        // Clean up any existing data before benchmark to ensure isolation
        let _ = self.drop_table();
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        // Clean up after benchmark iteration
        let _ = self.drop_table();
    }
}
