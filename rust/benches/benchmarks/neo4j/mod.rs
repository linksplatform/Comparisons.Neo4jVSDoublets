//! # Neo4j Benchmarks
//!
//! Every operation runs twice, once per transaction mode:
//!
//! | Benchmark name         | Mode                                                  |
//! |------------------------|-------------------------------------------------------|
//! | `Neo4j_NonTransaction` | every statement is an auto-commit transaction         |
//! | `Neo4j_Transaction`    | one explicit transaction per iteration, commit timed  |
//!
//! See [`linksneo4j::neo4j_impl`] for the Cypher statement of every operation.

use criterion::Criterion;
use linksneo4j::{Benched, Mode, Neo4j};

use super::{Operation, neo4j_group};

mod create;
mod delete;
pub mod each;
mod update;

pub use create::create_links;
pub use delete::delete_links;
pub use each::*;
pub use update::update_links;

/// Runs `operation` on Neo4j in both transaction modes.
fn run(c: &mut Criterion, group_name: &str, operation: Operation<Neo4j<usize>>) {
    let mut group = neo4j_group(c, group_name);
    for (id, mode) in [
        ("Neo4j_NonTransaction", Mode::AutoCommit),
        ("Neo4j_Transaction", Mode::Transaction),
    ] {
        let mut store = Neo4j::setup(mode).expect("cannot connect to Neo4j");
        operation(&mut group, id, &mut store);
    }
    group.finish();
}
