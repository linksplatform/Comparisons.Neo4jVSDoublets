//! # Doublets Query (Each) Benchmarks
//!
//! This module contains benchmarks for querying/iterating links in Doublets.
//!
//! ## Query Implementation
//!
//! | Query       | Implementation                                    |
//! |-------------|---------------------------------------------------|
//! | All         | Sequential array iteration                        |
//! | Identity    | Direct array access: `links[id]`                  |
//! | Concrete    | Index tree lookup + filter                        |
//! | Outgoing    | Source index tree traversal                       |
//! | Incoming    | Target index tree traversal                       |

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
