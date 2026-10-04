//! # Neo4j Query (Each) Benchmarks
//!
//! Every query is one `MATCH (l:Link) WHERE ... RETURN ...` statement; the
//! `WHERE` clause contains one condition per constrained part of the link.

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
