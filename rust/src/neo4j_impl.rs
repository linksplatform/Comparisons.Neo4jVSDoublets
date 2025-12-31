//! # Neo4j Implementation
//!
//! This module documents how Neo4j implements the common `Doublets<T>` interface.
//! Each operation is implemented using Cypher queries executed via HTTP API.
//!
//! ## Connection Setup
//!
//! ```rust,ignore
//! let client = Client::new("bolt://localhost:7687", "neo4j", "password")?;
//! ```
//!
//! ## Database Schema
//!
//! Neo4j stores links as nodes with the following structure:
//!
//! ```cypher
//! // Node label
//! :Link
//!
//! // Properties
//! {
//!     id: Integer,      // Unique link identifier
//!     source: Integer,  // Source link reference
//!     target: Integer   // Target link reference
//! }
//! ```
//!
//! ## Indexes Created on Setup
//!
//! ```cypher
//! CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE
//! CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)
//! CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)
//! ```
//!
//! ## Operations
//!
//! ### Create Point Link
//!
//! Interface method: `store.create_point()`
//!
//! ```cypher
//! CREATE (l:Link {id: $id, source: 0, target: 0})
//! ```
//!
//! - Generates next ID using atomic counter
//! - Creates node with source=0, target=0 (point link semantics)
//! - HTTP POST to `/db/neo4j/tx/commit`
//!
//! ### Update Link
//!
//! Interface method: `store.update(id, source, target)`
//!
//! ```cypher
//! // First, get old values:
//! MATCH (l:Link {id: $id}) RETURN l.source, l.target
//!
//! // Then, update:
//! MATCH (l:Link {id: $id}) SET l.source = $source, l.target = $target
//! ```
//!
//! - Two HTTP requests: one to read old values, one to update
//! - Returns both old and new link state
//!
//! ### Delete Link
//!
//! Interface method: `store.delete(id)`
//!
//! ```cypher
//! // First, get old values:
//! MATCH (l:Link {id: $id}) RETURN l.source, l.target
//!
//! // Then, delete:
//! MATCH (l:Link {id: $id}) DELETE l
//! ```
//!
//! - Two HTTP requests: one to read old values, one to delete
//! - Returns deleted link state
//!
//! ### Query All Links (Each All)
//!
//! Interface method: `store.each(handler)` or `store.each_by([any, any, any], handler)`
//!
//! ```cypher
//! MATCH (l:Link) RETURN l.id, l.source, l.target
//! ```
//!
//! - Full table scan
//! - Returns all nodes with :Link label
//!
//! ### Query by ID (Each Identity)
//!
//! Interface method: `store.each_by([id, any, any], handler)`
//!
//! ```cypher
//! MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses unique constraint index on `id`
//! - Time complexity: O(log n) index lookup + network overhead
//!
//! ### Query by Source (Each Outgoing)
//!
//! Interface method: `store.each_by([any, source, any], handler)`
//!
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses index on `source`
//! - Returns all outgoing edges from a node
//!
//! ### Query by Target (Each Incoming)
//!
//! Interface method: `store.each_by([any, any, target], handler)`
//!
//! ```cypher
//! MATCH (l:Link) WHERE l.target = $target RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses index on `target`
//! - Returns all incoming edges to a node
//!
//! ### Query by Source AND Target (Each Concrete)
//!
//! Interface method: `store.each_by([any, source, target], handler)`
//!
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source AND l.target = $target
//! RETURN l.id, l.source, l.target
//! ```
//!
//! - Uses one of the indexes (source or target), then filters
//! - Returns links matching both constraints
//!
//! ### Count Links
//!
//! Interface method: `store.count_links(query)`
//!
//! ```cypher
//! // Count all:
//! MATCH (l:Link) RETURN count(l)
//!
//! // Count by constraint:
//! MATCH (l:Link) WHERE l.source = $s AND l.target = $t RETURN count(l)
//! ```
//!
//! ### Get Link by ID
//!
//! Interface method: `store.get_link(id)`
//!
//! ```cypher
//! MATCH (l:Link {id: $id}) RETURN l.source, l.target
//! ```
//!
//! ## Cleanup Operations
//!
//! ```cypher
//! // Drop all links (used between benchmark iterations):
//! MATCH (l:Link) DETACH DELETE l
//! ```
//!
//! ## Performance Characteristics
//!
//! | Operation       | Time Complexity      | Notes                              |
//! |-----------------|----------------------|------------------------------------|
//! | Create          | O(log n) + network   | Index updates + HTTP overhead      |
//! | Update          | O(log n) + 2*network | Two HTTP requests required         |
//! | Delete          | O(log n) + 2*network | Two HTTP requests required         |
//! | Each All        | O(n) + network       | Full scan                          |
//! | Each Identity   | O(log n) + network   | Unique index lookup                |
//! | Each Outgoing   | O(log n + k) + network | Index lookup + k results         |
//! | Each Incoming   | O(log n + k) + network | Index lookup + k results         |
//! | Each Concrete   | O(log n + k) + network | Index + filter                   |

// This is a documentation-only module
