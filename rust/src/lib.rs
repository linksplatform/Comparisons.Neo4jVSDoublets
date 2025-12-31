#![feature(allocator_api, generic_associated_types)]

//! # Comparisons.Neo4jVSDoublets
//!
//! This crate provides a unified interface for comparing Neo4j and Doublets
//! database operations. It defines a common trait that both databases implement,
//! making it crystal clear what logic is being benchmarked against what.
//!
//! ## Detailed Implementation Documentation
//!
//! For detailed documentation on how each database implements the common interface,
//! see the separate implementation modules:
//!
//! - **[`doublets_impl`]** - How Doublets implements each operation with direct memory access
//! - **[`neo4j_impl`]** - How Neo4j implements each operation with Cypher queries
//!
//! These modules provide side-by-side comparison of the same logic implemented
//! in both databases.
//!
//! ## The Common Interface
//!
//! Both Neo4j and Doublets implement the same operations through the [`Doublets<T>`]
//! trait from the `doublets` crate. This benchmark compares how each database
//! implements these core link operations:
//!
//! | Operation       | Method                  | Description                                    |
//! |-----------------|-------------------------|------------------------------------------------|
//! | Create          | `create_point()`        | Insert a point link (id = source = target)     |
//! | Update          | `update(id, src, tgt)`  | Modify source and target of existing link      |
//! | Delete          | `delete(id)`            | Remove a link by its ID                        |
//! | Each All        | `each(handler)`         | Iterate all links matching `[*, *, *]`         |
//! | Each Identity   | `each_by([id,*,*], h)`  | Find links by ID constraint                    |
//! | Each Concrete   | `each_by([*,s,t], h)`   | Find links by source AND target                |
//! | Each Outgoing   | `each_by([*,s,*], h)`   | Find links by source (outgoing edges)          |
//! | Each Incoming   | `each_by([*,*,t], h)`   | Find links by target (incoming edges)          |
//!
//! ## Benchmarked Implementations
//!
//! | Implementation                | Backend Type            | Description                           |
//! |-------------------------------|-------------------------|---------------------------------------|
//! | `Doublets_United_Volatile`    | In-memory (unit store)  | Fast, RAM-only storage                |
//! | `Doublets_United_NonVolatile` | File-mapped (unit)      | Persistent, memory-mapped file        |
//! | `Doublets_Split_Volatile`     | In-memory (split store) | Separate data/index in RAM            |
//! | `Doublets_Split_NonVolatile`  | File-mapped (split)     | Separate data/index files             |
//! | `Neo4j_NonTransaction`        | HTTP auto-commit        | Each operation is separate request    |
//! | `Neo4j_Transaction`           | HTTP auto-commit (same) | Uses transaction wrapper (same impl)  |
//!
//! ## How the Benchmark Works
//!
//! Each benchmark iteration:
//! 1. Sets up a fresh storage backend (via [`Benched::setup`])
//! 2. Creates [`BACKGROUND_LINKS`] links to simulate a populated database
//! 3. Executes the benchmarked operation ([`LINK_COUNT`] times for CRUD operations)
//! 4. Cleans up via [`Benched::unfork`]
//!
//! The [`Benched`] trait provides the setup/teardown lifecycle, while [`Doublets<T>`]
//! provides the actual database operations being measured.

#[macro_export]
macro_rules! bench {
    {|$fork:ident| as $B:ident { $($body:tt)* }} => {
        (move |bencher: &mut criterion::Bencher, benched: &mut _| {
            bencher.iter_custom(|iters| {
                let mut __bench_duration = Duration::ZERO;
                macro_rules! elapsed {
                    {$expr:expr} => {{
                        let __instant = Instant::now();
                        let __ret = {$expr};
                        __bench_duration += __instant.elapsed();
                        __ret
                    }};
                }
                crate::tri! {
                    use linksneo4j::BACKGROUND_LINKS;
                    for _iter in 0..iters {
                        let mut $fork: Fork<$B> = Benched::fork(&mut *benched);
                        for _ in 0..BACKGROUND_LINKS {
                            let _ = $fork.create_point()?;
                        }
                        $($body)*
                    }
                }
                __bench_duration
            });
        })
    }
}

use std::{alloc::Global, error, fs::File, io, result};

pub use benched::Benched;
pub use client::Client;
use doublets::{
    data::LinkType,
    mem::{Alloc, FileMapped},
    split::{self, DataPart, IndexPart},
    unit::{self, LinkPart},
};
pub use exclusive::Exclusive;
pub use fork::Fork;
pub use transaction::Transaction;

mod benched;
mod client;
pub mod doublets_impl;
mod exclusive;
mod fork;
pub mod neo4j_impl;
mod transaction;

pub type Result<T, E = Box<dyn error::Error + Sync + Send>> = result::Result<T, E>;

/// Number of background links to create before each benchmark iteration.
/// This simulates a database with existing data.
pub const BACKGROUND_LINKS: usize = 10;

/// Number of links to create/delete/update in each benchmark operation.
/// Can be configured via BENCHMARK_LINK_COUNT environment variable.
/// Defaults to 10 for faster iteration in pull requests, but should be set to 1000 for main branch benchmarks.
pub fn link_count() -> usize {
    std::env::var("BENCHMARK_LINK_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// Lazy static to cache the link count value
pub use once_cell::sync::Lazy;
pub static LINK_COUNT: Lazy<usize> = Lazy::new(link_count);

/// Connect to Neo4j database
pub fn connect<T: LinkType>() -> Result<Client<T>> {
    // Default Neo4j connection parameters
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
    Client::new(&uri, &user, &password)
}

pub fn map_file<T: Default>(filename: &str) -> io::Result<FileMapped<T>> {
    let file = File::options()
        .create(true)
        .write(true)
        .read(true)
        .open(filename)?;
    FileMapped::new(file)
}

pub trait Sql {
    fn create_table(&mut self) -> Result<()>;
    fn drop_table(&mut self) -> Result<()>;
}

// ============================================================================
// BENCHMARKED STORAGE TYPE ALIASES
// ============================================================================
//
// These type aliases provide clear names for each storage implementation being
// benchmarked. All types implement the `Doublets<T>` trait, which provides the
// common interface for link operations.

/// Doublets United (unit) store with volatile (in-memory) storage.
///
/// ## Storage Structure
/// Each link is stored as a single contiguous unit containing `(id, source, target)`.
/// Uses direct array indexing for O(1) access by link ID.
///
/// ## Cypher equivalent (what Neo4j does for the same operation)
/// ```cypher
/// // Create point link:
/// CREATE (l:Link {id: $id, source: $id, target: $id})
///
/// // Read link by ID:
/// MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target
/// ```
pub type DoubletsUnitedVolatile<T = usize> = unit::Store<T, Alloc<LinkPart<T>, Global>>;

/// Doublets United (unit) store with non-volatile (file-mapped) storage.
///
/// Same as [`DoubletsUnitedVolatile`] but uses memory-mapped files for persistence.
/// Changes are automatically synced to disk.
pub type DoubletsUnitedNonVolatile<T = usize> = unit::Store<T, FileMapped<LinkPart<T>>>;

/// Doublets Split store with volatile (in-memory) storage.
///
/// ## Storage Structure
/// Separates data and index into different memory regions:
/// - **DataPart**: Contains `(source, target)` pairs
/// - **IndexPart**: Contains trees for fast source/target lookups
///
/// This separation improves cache efficiency for index-heavy operations.
pub type DoubletsSplitVolatile<T = usize> =
    split::Store<T, Alloc<DataPart<T>, Global>, Alloc<IndexPart<T>, Global>>;

/// Doublets Split store with non-volatile (file-mapped) storage.
///
/// Same as [`DoubletsSplitVolatile`] but uses memory-mapped files for persistence.
/// Data and index are stored in separate files.
pub type DoubletsSplitNonVolatile<T = usize> =
    split::Store<T, FileMapped<DataPart<T>>, FileMapped<IndexPart<T>>>;

/// Neo4j client (non-transactional mode).
///
/// ## Implementation Details
/// Uses HTTP API to execute Cypher queries against Neo4j. Each operation makes
/// a separate HTTP request to `/db/neo4j/tx/commit` endpoint.
///
/// ## Cypher commands used for benchmarked operations
/// ```cypher
/// // Create point link:
/// CREATE (l:Link {id: $id, source: 0, target: 0})
///
/// // Update link:
/// MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target
///
/// // Delete link:
/// MATCH (l:Link {id: $id}) DELETE l
///
/// // Query by ID (Each Identity):
/// MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target
///
/// // Query by source (Each Outgoing):
/// MATCH (l:Link) WHERE l.source = $source RETURN l.id, l.source, l.target
///
/// // Query by target (Each Incoming):
/// MATCH (l:Link) WHERE l.target = $target RETURN l.id, l.source, l.target
///
/// // Query by source AND target (Each Concrete):
/// MATCH (l:Link) WHERE l.source = $s AND l.target = $t RETURN l.id, l.source, l.target
///
/// // Query all (Each All):
/// MATCH (l:Link) RETURN l.id, l.source, l.target
/// ```
pub type Neo4jNonTransaction<T = usize> = Exclusive<Client<T>>;

/// Neo4j transaction wrapper.
///
/// Uses the same HTTP API as [`Neo4jNonTransaction`] since the `/db/neo4j/tx/commit`
/// endpoint auto-commits each request. The transaction wrapper exists for API
/// compatibility and to measure any overhead from the wrapper itself.
pub type Neo4jTransaction<'a, T = usize> = Exclusive<Transaction<'a, T>>;
