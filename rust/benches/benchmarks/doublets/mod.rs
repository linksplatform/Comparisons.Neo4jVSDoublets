//! # Doublets Benchmark Implementations
//!
//! This module contains all benchmark implementations for Doublets.
//! Each benchmark tests a specific database operation using direct memory access
//! to specialized data structures.
//!
//! ## Benchmarked Operations
//!
//! | Benchmark       | Implementation                                  |
//! |-----------------|-------------------------------------------------|
//! | `create_links`  | Allocate ID, write to memory, update indexes    |
//! | `delete_links`  | Remove from indexes, mark slot as free          |
//! | `update_links`  | Update storage, re-index if values changed      |
//! | `each_all`      | Sequential array iteration                      |
//! | `each_identity` | Direct array access: `links[id]`                |
//! | `each_concrete` | Index tree lookup + filter                      |
//! | `each_outgoing` | Source index tree traversal                     |
//! | `each_incoming` | Target index tree traversal                     |
//!
//! ## Storage Backends Tested
//!
//! - `Doublets_United_Volatile` - In-memory unit storage
//! - `Doublets_United_NonVolatile` - File-mapped unit storage
//! - `Doublets_Split_Volatile` - In-memory split storage (separate data/index)
//! - `Doublets_Split_NonVolatile` - File-mapped split storage

mod create;
mod delete;
pub mod each;
mod update;

pub use create::create_links;
pub use delete::delete_links;
pub use each::*;
pub use update::update_links;
