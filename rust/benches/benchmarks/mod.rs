//! # Benchmark Implementations
//!
//! This module contains all benchmark implementations comparing Neo4j and Doublets.
//! Each benchmark tests a specific database operation across all storage backends.
//!
//! ## Module Structure
//!
//! The benchmarks are split into two separate modules for clear comparison:
//!
//! - **[`neo4j`]** - All Neo4j benchmarks using Cypher queries via HTTP API
//! - **[`doublets`]** - All Doublets benchmarks using direct memory access
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
//! ### Doublets (4 variants)
//! - `Doublets_United_Volatile` - In-memory unit storage
//! - `Doublets_United_NonVolatile` - File-mapped unit storage
//! - `Doublets_Split_Volatile` - In-memory split storage (separate data/index)
//! - `Doublets_Split_NonVolatile` - File-mapped split storage
//!
//! ### Neo4j (2 variants)
//! - `Neo4j_NonTransaction` - Direct HTTP API calls
//! - `Neo4j_Transaction` - Transaction wrapper (same underlying implementation)

pub mod doublets;
pub mod neo4j;

// Re-export all Neo4j benchmarks with neo4j_ prefix
pub use neo4j::create_links as neo4j_create_links;
pub use neo4j::delete_links as neo4j_delete_links;
pub use neo4j::each_all as neo4j_each_all;
pub use neo4j::each_concrete as neo4j_each_concrete;
pub use neo4j::each_identity as neo4j_each_identity;
pub use neo4j::each_incoming as neo4j_each_incoming;
pub use neo4j::each_outgoing as neo4j_each_outgoing;
pub use neo4j::update_links as neo4j_update_links;

// Re-export all Doublets benchmarks with doublets_ prefix
pub use self::doublets::create_links as doublets_create_links;
pub use self::doublets::delete_links as doublets_delete_links;
pub use self::doublets::each_all as doublets_each_all;
pub use self::doublets::each_concrete as doublets_each_concrete;
pub use self::doublets::each_identity as doublets_each_identity;
pub use self::doublets::each_incoming as doublets_each_incoming;
pub use self::doublets::each_outgoing as doublets_each_outgoing;
pub use self::doublets::update_links as doublets_update_links;
