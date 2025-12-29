//! # Query (Each) Benchmarks
//!
//! This module contains benchmarks for querying/iterating links in both
//! Doublets and Neo4j databases.
//!
//! ## Common Interface Method
//!
//! Both implementations use the `Links<T>::each_by(constraint, handler)` method:
//! ```rust,ignore
//! // Query links matching a constraint pattern [id, source, target]
//! // Use `any` (usually 0 or max value) as wildcard
//! store.each_by([any, source, any], |link| Flow::Continue);
//! ```
//!
//! ## Query Types
//!
//! | Query       | Constraint       | Description                |
//! |-------------|------------------|----------------------------|
//! | All         | `[*, *, *]`      | Return all links           |
//! | Identity    | `[id, *, *]`     | Find by primary key        |
//! | Concrete    | `[*, src, tgt]`  | Find by source AND target  |
//! | Outgoing    | `[*, src, *]`    | Find by source (edges from)|
//! | Incoming    | `[*, *, tgt]`    | Find by target (edges to)  |
//!
//! ## Neo4j Cypher Equivalents
//!
//! | Query       | Cypher                                                          |
//! |-------------|-----------------------------------------------------------------|
//! | All         | `MATCH (l:Link) RETURN l.id, l.source, l.target`                |
//! | Identity    | `MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target`      |
//! | Concrete    | `MATCH (l:Link) WHERE l.source = $s AND l.target = $t RETURN...`|
//! | Outgoing    | `MATCH (l:Link) WHERE l.source = $source RETURN...`             |
//! | Incoming    | `MATCH (l:Link) WHERE l.target = $target RETURN...`             |

mod all;
mod concrete;
mod identity;
mod incoming;
mod outgoing;

pub use all::each_all;
pub use concrete::each_concrete;
pub use identity::each_identity;
pub use incoming::each_incoming;
pub use outgoing::each_outgoing;
