//! # Benchmark Implementations
//!
//! This module contains all benchmark implementations comparing Neo4j and Doublets.
//! Each benchmark tests a specific database operation across all storage backends.
//!
//! ## Benchmarked Operations
//!
//! | Benchmark       | Operation                                      | What it measures                    |
//! |-----------------|------------------------------------------------|-------------------------------------|
//! | `create_links`  | Insert point links (id = source = target)      | Write performance                   |
//! | `delete_links`  | Remove links by ID                             | Delete performance                  |
//! | `update_links`  | Modify source/target of existing links         | Update performance                  |
//! | `each_all`      | Query all links `[*, *, *]`                    | Full scan performance               |
//! | `each_identity` | Query by ID `[id, *, *]`                       | Primary key lookup                  |
//! | `each_concrete` | Query by source+target `[*, src, tgt]`         | Composite index lookup              |
//! | `each_outgoing` | Query by source `[*, src, *]`                  | Source index lookup                 |
//! | `each_incoming` | Query by target `[*, *, tgt]`                  | Target index lookup                 |
//!
//! ## Storage Backends Tested
//!
//! Each benchmark runs against these 6 storage implementations:
//!
//! ### Doublets (4 variants)
//! - `Doublets_United_Volatile` - In-memory unit storage
//! - `Doublets_United_NonVolatile` - File-mapped unit storage
//! - `Doublets_Split_Volatile` - In-memory split storage (separate data/index)
//! - `Doublets_Split_NonVolatile` - File-mapped split storage
//!
//! ### Neo4j (2 variants)
//! - `Neo4j_NonTransaction` - Direct HTTP API calls
//! - `Neo4j_Transaction` - Transaction wrapper (same underlying implementation)

mod create;
mod delete;
mod each;
mod update;

pub use create::create_links;
pub use delete::delete_links;
pub use each::*;
pub use update::update_links;
