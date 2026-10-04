//! # Neo4j Implementation
//!
//! [`Neo4j`] implements the same [`Doublets`] interface as the Doublets stores,
//! so every benchmark calls exactly the same methods on both databases.
//!
//! ## Driver and connection
//!
//! Neo4j is reached through [neo4rs](https://github.com/neo4j-labs/neo4rs),
//! the Bolt driver maintained by Neo4j Labs. Neo4j does not publish an
//! official Rust driver.
//!
//! - The connection pool is opened once in [`Neo4j::connect`] and reused by
//!   all operations, so no TCP connection is opened per operation.
//! - All settings of the driver and of the server are defaults.
//!
//! ## Data model
//!
//! A link is a node with three integer properties:
//!
//! ```cypher
//! (:Link {id: Integer, source: Integer, target: Integer})
//! ```
//!
//! Links are found by the `id` property, which is backed by a uniqueness
//! constraint (and therefore by an index). The internal node id (`elementId`)
//! is not used, because Neo4j assigns it itself and may reuse it after a node
//! is deleted, while a link id must be chosen by the caller and stay stable.
//!
//! ```cypher
//! CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE
//! CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)
//! CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)
//! ```
//!
//! ## One statement per operation
//!
//! | Operation      | Cypher                                                                        |
//! |----------------|-------------------------------------------------------------------------------|
//! | `create_point` | `CREATE (:Link {id: $id, source: $id, target: $id})`                          |
//! | `update`       | `MATCH (l:Link {id: $id}) WITH l, l.source AS source, l.target AS target SET l.source = $new_source, l.target = $new_target RETURN source, target` |
//! | `delete`       | `MATCH (l:Link {id: $id}) WITH l, l.source AS source, l.target AS target DELETE l RETURN source, target` |
//! | `each_by`      | `MATCH (l:Link) WHERE <constraints> RETURN l.id AS id, l.source AS source, l.target AS target` |
//! | `count_by`     | `MATCH (l:Link) WHERE <constraints> RETURN count(l) AS count`                 |
//!
//! `update` and `delete` return the previous `source` and `target` from the
//! same statement, because the [`Doublets`] interface reports the link state
//! before and after each change.
//!
//! New ids come from a counter kept by the client, as with a database
//! sequence: `1, 2, 3, ...`. Doublets also assigns `1, 2, 3, ...` to the links
//! of an empty store. As in Doublets, deleting the link with the highest id
//! makes that id the next one to be created. Ids of other deleted links are not
//! reused (Doublets reuses them too); the benchmarks only delete the links with
//! the highest ids, so both databases always assign the same ids.
//!
//! ## Transaction modes
//!
//! | [`Mode`]                 | What happens                                                         |
//! |--------------------------|----------------------------------------------------------------------|
//! | [`Mode::AutoCommit`]     | Every statement is its own transaction, committed by the server.     |
//! | [`Mode::Transaction`]    | All statements between [`Neo4j::begin`] and [`Neo4j::commit`] run in one explicit transaction. |
//!
//! Errors returned by Neo4j are never ignored: methods that return `Result`
//! return them as [`Error::Other`], and methods of the [`Links`] trait that
//! cannot return an error (`each_links`, `count_links`) panic, which stops the
//! benchmark.

use std::{
    error,
    sync::{Mutex, MutexGuard},
};

use doublets::{
    Doublets, Error, Link, Links,
    data::{Flow, LinkReference, LinksConstants, ReadHandler, WriteHandler},
};
use neo4rs::{Graph, Query, Row, Txn, query};
use tokio::runtime::Runtime;

/// How statements are grouped into transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Every statement is a separate, automatically committed transaction.
    AutoCommit,
    /// Statements run in one explicit transaction until [`Neo4j::commit`].
    Transaction,
}

/// A links store kept in a Neo4j database.
pub struct Neo4j<T: LinkReference> {
    /// Runs the asynchronous driver calls to completion on the calling thread.
    runtime: Runtime,
    graph: Graph,
    mode: Mode,
    /// The open explicit transaction, used only in [`Mode::Transaction`].
    transaction: Mutex<Option<Txn>>,
    constants: LinksConstants<T>,
    next_id: T,
}

/// Number of links created or deleted per transaction by
/// [`Neo4j::create_points`] and [`Neo4j::clear`].
pub const BATCH_SIZE: usize = 10_000;

/// Result of a driver call.
type Neo4jResult<T> = Result<T, neo4rs::Error>;

impl<T: LinkReference> Neo4j<T> {
    /// Opens the connection pool and creates the schema if it does not exist.
    pub fn connect(uri: &str, user: &str, password: &str, mode: Mode) -> crate::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let graph = runtime.block_on(Graph::new(uri, user, password))?;
        let store = Self {
            runtime,
            graph,
            mode,
            transaction: Mutex::new(None),
            constants: LinksConstants::new(),
            next_id: T::from_byte(1),
        };
        for statement in [
            "CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE",
            "CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)",
            "CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)",
        ] {
            store.run_auto_commit(query(statement))?;
        }
        store.run_auto_commit(query("CALL db.awaitIndexes()"))?;
        Ok(store)
    }

    /// Opens the connection described by the `NEO4J_URI`, `NEO4J_USER` and
    /// `NEO4J_PASSWORD` environment variables (defaults:
    /// `bolt://localhost:7687`, `neo4j`, `password`).
    pub fn connect_from_env(mode: Mode) -> crate::Result<Self> {
        let var = |name, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_owned());
        Self::connect(
            &var("NEO4J_URI", "bolt://localhost:7687"),
            &var("NEO4J_USER", "neo4j"),
            &var("NEO4J_PASSWORD", "password"),
            mode,
        )
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Creates `count` point links with the next ids, committing every
    /// [`BATCH_SIZE`] links, so that millions of links do not have to fit into
    /// the memory of one transaction.
    ///
    /// Used to prepare the background links outside of the measured time.
    pub fn create_points(&mut self, count: usize) -> crate::Result<()> {
        let first = id_to_i64(self.next_id);
        let last = first + count as i64 - 1;
        self.run_auto_commit(
            query(&format!(
                "UNWIND range($first, $last) AS id \
                 CALL (id) {{ CREATE (:Link {{id: id, source: id, target: id}}) }} \
                 IN TRANSACTIONS OF {BATCH_SIZE} ROWS"
            ))
            .param("first", first)
            .param("last", last),
        )?;
        self.next_id = id_from_i64(last + 1);
        Ok(())
    }

    /// Starts an explicit transaction when the store is in [`Mode::Transaction`].
    pub fn begin(&mut self) -> crate::Result<()> {
        if self.mode == Mode::Transaction {
            let transaction = self.runtime.block_on(self.graph.start_txn())?;
            *self.transaction() = Some(transaction);
        }
        Ok(())
    }

    /// Commits the explicit transaction started by [`Neo4j::begin`], if any.
    pub fn commit(&mut self) -> crate::Result<()> {
        if let Some(transaction) = self.transaction().take() {
            self.runtime.block_on(transaction.commit())?;
        }
        Ok(())
    }

    /// Rolls back the open transaction, if any, and deletes all links,
    /// committing every [`BATCH_SIZE`] links.
    pub fn clear(&mut self) -> crate::Result<()> {
        if let Some(transaction) = self.transaction().take() {
            self.runtime.block_on(transaction.rollback())?;
        }
        self.run_auto_commit(query(&format!(
            "MATCH (l:Link) CALL (l) {{ DELETE l }} IN TRANSACTIONS OF {BATCH_SIZE} ROWS"
        )))?;
        self.next_id = T::from_byte(1);
        Ok(())
    }

    fn transaction(&self) -> MutexGuard<'_, Option<Txn>> {
        self.transaction
            .lock()
            .expect("transaction mutex is poisoned")
    }

    fn run_auto_commit(&self, query: Query) -> Neo4jResult<()> {
        self.runtime.block_on(self.graph.run(query))
    }

    /// Runs `query` in the open transaction (or as an auto-commit
    /// transaction) and passes every returned row to `on_row`.
    // The lock is held only by this thread while `block_on` runs the query on
    // the store's own single-threaded runtime, so no other task can wait for it.
    #[allow(clippy::await_holding_lock)]
    fn fetch(&self, query: Query, mut on_row: impl FnMut(Row)) -> Neo4jResult<()> {
        self.runtime.block_on(async {
            match self.transaction().as_mut() {
                Some(transaction) => {
                    let mut rows = transaction.execute(query).await?;
                    while let Some(row) = rows.next(transaction.handle()).await? {
                        on_row(row);
                    }
                }
                None => {
                    let mut rows = self.graph.execute(query).await?;
                    while let Some(row) = rows.next().await? {
                        on_row(row);
                    }
                }
            }
            Ok(())
        })
    }

    /// Runs `query` and returns its rows.
    fn fetch_all(&self, query: Query) -> Neo4jResult<Vec<Row>> {
        let mut rows = Vec::new();
        self.fetch(query, |row| rows.push(row))?;
        Ok(rows)
    }

    /// Builds `MATCH (l:Link) WHERE ...` for a doublets query
    /// (`[]`, `[id]` or `[id, source, target]`, where `any` matches everything).
    fn match_query(&self, query: &[T], returns: &str) -> Query {
        let any = self.constants.any;
        let names = ["id", "source", "target"];
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        for (name, &value) in names.iter().zip(query) {
            if value != any {
                conditions.push(format!("l.{name} = ${name}"));
                params.push((*name, id_to_i64(value)));
            }
        }
        let filter = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };
        params.into_iter().fold(
            neo4rs::query(&format!("MATCH (l:Link){filter} RETURN {returns}")),
            |q, (name, value)| q.param(name, value),
        )
    }
}

impl<T: LinkReference> Links<T> for Neo4j<T> {
    fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    fn count_links(&self, query: &[T]) -> T {
        let rows = self
            .fetch_all(self.match_query(query, "count(l) AS count"))
            .expect("Neo4j count query failed");
        id_from_i64(rows[0].get("count").expect("count column"))
    }

    fn create_links(
        &mut self,
        _query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let id = self.next_id;
        self.fetch_all(
            query("CREATE (:Link {id: $id, source: 0, target: 0})").param("id", id_to_i64(id)),
        )
        .map_err(other)?;
        self.next_id = id + T::from_byte(1);
        let null = self.constants.null;
        Ok(handler(Link::nothing(), Link::new(id, null, null)))
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        let mut flow = Flow::Continue;
        self.fetch(
            self.match_query(query, "l.id AS id, l.source AS source, l.target AS target"),
            |row| {
                // The remaining rows are still read, so the result stream is
                // fully consumed before the next statement is sent.
                if flow.is_continue() {
                    flow = handler(link_from_row(&row));
                }
            },
        )
        .expect("Neo4j read query failed");
        flow
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let (id, source, target) = (query[0], change[1], change[2]);
        let rows = self
            .fetch_all(
                neo4rs::query(
                    "MATCH (l:Link {id: $id}) \
                     WITH l, l.source AS source, l.target AS target \
                     SET l.source = $new_source, l.target = $new_target \
                     RETURN source, target",
                )
                .param("id", id_to_i64(id))
                .param("new_source", id_to_i64(source))
                .param("new_target", id_to_i64(target)),
            )
            .map_err(other)?;
        let old = rows.first().ok_or(Error::NotExists(id))?;
        let (old_source, old_target) = (column(old, "source"), column(old, "target"));
        Ok(handler(
            Link::new(id, old_source, old_target),
            Link::new(id, source, target),
        ))
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let id = query[0];
        let rows = self
            .fetch_all(
                neo4rs::query(
                    "MATCH (l:Link {id: $id}) \
                     WITH l, l.source AS source, l.target AS target \
                     DELETE l \
                     RETURN source, target",
                )
                .param("id", id_to_i64(id)),
            )
            .map_err(other)?;
        let old = rows.first().ok_or(Error::NotExists(id))?;
        let (old_source, old_target) = (column(old, "source"), column(old, "target"));
        if id + T::from_byte(1) == self.next_id {
            self.next_id = id;
        }
        Ok(handler(
            Link::new(id, old_source, old_target),
            Link::nothing(),
        ))
    }
}

impl<T: LinkReference> Doublets<T> for Neo4j<T> {
    fn get_link(&self, index: T) -> Option<Link<T>> {
        let mut found = None;
        self.each_links(&[index], &mut |link| {
            found = Some(link);
            Flow::Break
        });
        found
    }

    /// Creates the point link with one statement instead of the default
    /// `create` followed by `update`.
    fn create_point(&mut self) -> Result<T, Error<T>> {
        let id = self.next_id;
        self.fetch_all(
            query("CREATE (:Link {id: $id, source: $id, target: $id})").param("id", id_to_i64(id)),
        )
        .map_err(other)?;
        self.next_id = id + T::from_byte(1);
        Ok(id)
    }
}

fn link_from_row<T: LinkReference>(row: &Row) -> Link<T> {
    Link::new(
        column(row, "id"),
        column(row, "source"),
        column(row, "target"),
    )
}

fn column<T: LinkReference>(row: &Row, name: &str) -> T {
    id_from_i64(
        row.get(name)
            .unwrap_or_else(|e| panic!("column `{name}`: {e}")),
    )
}

/// Neo4j stores integers as 64-bit signed values.
fn id_to_i64<T: LinkReference>(id: T) -> i64 {
    id.try_into().expect("link id does not fit into i64")
}

fn id_from_i64<T: LinkReference>(id: i64) -> T {
    T::try_from(id).expect("Neo4j returned an id that does not fit into the link type")
}

fn other<T: LinkReference>(error: neo4rs::Error) -> Error<T> {
    Error::Other(Box::new(error) as Box<dyn error::Error + Send + Sync>)
}
