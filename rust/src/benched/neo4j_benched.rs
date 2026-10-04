//! # Neo4j Benched Implementation
//!
//! | Benchmark name         | [`Mode`]                | Transaction                              |
//! |------------------------|-------------------------|------------------------------------------|
//! | `Neo4j_NonTransaction` | [`Mode::AutoCommit`]    | one auto-commit transaction per statement |
//! | `Neo4j_Transaction`    | [`Mode::Transaction`]   | one explicit transaction per iteration   |
//!
//! The background links are created with a single `UNWIND` statement and
//! removed with:
//! ```cypher
//! MATCH (l:Link) DETACH DELETE l
//! ```

use doublets::data::LinkReference;

use super::Benched;
use crate::{Fork, Mode, Neo4j};

impl<T: LinkReference> Benched for Neo4j<T> {
    type Builder<'a> = Mode;

    fn setup(mode: Self::Builder<'_>) -> crate::Result<Self> {
        let mut store = Self::connect_from_env(mode)?;
        store.clear()?;
        Ok(store)
    }

    fn fork(&mut self, background_links: usize) -> crate::Result<Fork<'_, Self>> {
        self.create_points(background_links)?;
        Ok(Fork(self))
    }

    fn begin(&mut self) -> crate::Result<()> {
        Neo4j::begin(self)
    }

    fn commit(&mut self) -> crate::Result<()> {
        Neo4j::commit(self)
    }

    fn unfork(&mut self) -> crate::Result<()> {
        self.clear()
    }
}
