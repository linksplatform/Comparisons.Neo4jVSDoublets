//! # Neo4j Benchmark Implementations
//!
//! This module contains all benchmark implementations for Neo4j.
//! Each benchmark tests a specific database operation using Cypher queries
//! executed via HTTP API.
//!
//! ## Benchmarked Operations
//!
//! | Benchmark       | Cypher Query                                    |
//! |-----------------|------------------------------------------------|
//! | `create_links`  | `CREATE (l:Link {id: $id, source: 0, target: 0})`|
//! | `delete_links`  | `MATCH (l:Link {id: $id}) DELETE l`             |
//! | `update_links`  | `MATCH (l:Link {id: $id}) SET l.source=..., l.target=...`|
//! | `each_all`      | `MATCH (l:Link) RETURN l.id, l.source, l.target`|
//! | `each_identity` | `MATCH (l:Link {id: $id}) RETURN ...`           |
//! | `each_concrete` | `MATCH (l:Link) WHERE l.source=$s AND l.target=$t RETURN ...`|
//! | `each_outgoing` | `MATCH (l:Link) WHERE l.source=$source RETURN ...`|
//! | `each_incoming` | `MATCH (l:Link) WHERE l.target=$target RETURN ...`|
//!
//! ## Storage Backends Tested
//!
//! - `Neo4j_NonTransaction` - Direct HTTP API calls
//! - `Neo4j_Transaction` - Transaction wrapper (same underlying implementation)

mod create;
mod delete;
pub mod each;
mod update;

pub use create::create_links;
pub use delete::delete_links;
pub use each::*;
pub use update::update_links;
