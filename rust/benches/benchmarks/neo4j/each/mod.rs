//! # Neo4j Query (Each) Benchmarks
//!
//! This module contains benchmarks for querying/iterating links in Neo4j.
//!
//! ## Cypher Queries Used
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
