//! # Comparisons.Neo4jVSDoublets
//!
//! This crate measures how long basic link operations take when the links are
//! stored in Neo4j and when they are stored in Doublets.
//!
//! Both databases implement the same [`Doublets`](doublets::Doublets) trait,
//! and every benchmark calls the same trait methods on every backend:
//!
//! - **[`neo4j_impl`]** - how Neo4j implements each operation (Cypher via the
//!   neo4rs Bolt driver)
//! - **[`doublets_impl`]** - how Doublets implements each operation (direct
//!   access to its in-process data structures)
//!
//! ## Operations
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
//! | Implementation                | Backend                          | Durability of a finished operation        |
//! |-------------------------------|----------------------------------|-------------------------------------------|
//! | `Doublets_United_Volatile`    | in-process, RAM, unit store      | none (RAM only)                           |
//! | `Doublets_United_NonVolatile` | in-process, memory-mapped file   | in the OS page cache, no `fsync`          |
//! | `Doublets_Split_Volatile`     | in-process, RAM, split store     | none (RAM only)                           |
//! | `Doublets_Split_NonVolatile`  | in-process, memory-mapped files  | in the OS page cache, no `fsync`          |
//! | `Neo4j_NonTransaction`        | Neo4j server over Bolt           | committed (auto-commit per statement)     |
//! | `Neo4j_Transaction`           | Neo4j server over Bolt           | committed once per iteration              |
//! | `Neo4j_Batch`                 | Neo4j server over Bolt           | one statement with all `N` operations     |
//!
//! ## How the Benchmark Works
//!
//! For every operation and store:
//! 1. [`Benched::fork`] creates [`background_links`] point links (not measured)
//! 2. every iteration runs the operation [`benchmark_links`] times between
//!    [`Benched::begin`] and [`Benched::commit`] (measured), and then undoes
//!    its changes (not measured), so all iterations start from the same links
//! 3. [`Benched::unfork`] removes all links (not measured)

use std::{env, error, fs::File, io, result};

pub use benched::Benched;
use doublets::{
    mem::{FileMapped, Global},
    split::{self, DataPart, IndexPart},
    unit::{self, LinkPart},
};
pub use fork::Fork;
pub use neo4j_impl::{Mode, Neo4j};

mod benched;
pub mod doublets_impl;
mod fork;
pub mod neo4j_impl;

pub type Result<T, E = Box<dyn error::Error + Sync + Send>> = result::Result<T, E>;

/// Number of point links that exist before the measured operations start.
///
/// Configurable via the `BENCHMARK_BACKGROUND_LINKS` environment variable.
pub fn background_links() -> usize {
    env_usize("BENCHMARK_BACKGROUND_LINKS", 1000)
}

/// Number of links that each benchmark iteration creates, updates, deletes or
/// looks up.
///
/// Configurable via the `BENCHMARK_LINKS` environment variable. Update and
/// Delete work on existing background links, so this value must not exceed
/// [`background_links`].
pub fn benchmark_links() -> usize {
    let links = env_usize("BENCHMARK_LINKS", 100);
    assert!(
        links <= background_links(),
        "BENCHMARK_LINKS ({links}) must not exceed BENCHMARK_BACKGROUND_LINKS ({})",
        background_links()
    );
    links
}

fn env_usize(name: &str, default: usize) -> usize {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be a non-negative integer, got `{value}`")),
        Err(_) => default,
    }
}

pub fn map_file<T: Default>(filename: &str) -> io::Result<FileMapped<T>> {
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .read(true)
        .open(filename)?;
    FileMapped::new(file)
}

// ============================================================================
// BENCHMARKED STORAGE TYPE ALIASES
// ============================================================================

/// Doublets United (unit) store in RAM.
///
/// Each array element holds `(source, target)` of one link together with its
/// index tree nodes; the id is the position in the array.
pub type DoubletsUnitedVolatile<T = usize> = unit::Store<T, Global<LinkPart<T>>>;

/// Doublets United (unit) store in a memory-mapped file.
pub type DoubletsUnitedNonVolatile<T = usize> = unit::Store<T, FileMapped<LinkPart<T>>>;

/// Doublets Split store in RAM.
///
/// Data (`source`, `target`) and index trees are kept in separate memory
/// regions.
pub type DoubletsSplitVolatile<T = usize> =
    split::Store<T, Global<DataPart<T>>, Global<IndexPart<T>>>;

/// Doublets Split store in two memory-mapped files.
pub type DoubletsSplitNonVolatile<T = usize> =
    split::Store<T, FileMapped<DataPart<T>>, FileMapped<IndexPart<T>>>;
