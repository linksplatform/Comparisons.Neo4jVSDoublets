//! # Doublets Implementation
//!
//! This module documents how Doublets implements the common `Doublets<T>` interface.
//! Each operation is implemented using direct memory access to specialized data structures.
//!
//! ## Storage Types
//!
//! ### United (Unit) Store
//!
//! ```rust,ignore
//! // Volatile (in-memory):
//! let store = unit::Store::new(Alloc::new(Global))?;
//!
//! // Non-volatile (file-mapped):
//! let store = unit::Store::new(FileMapped::new(file)?)?;
//! ```
//!
//! Each link is stored as a contiguous `LinkPart<T>`:
//! ```text
//! +--------+--------+--------+
//! |   id   | source | target |
//! +--------+--------+--------+
//! ```
//!
//! ### Split Store
//!
//! ```rust,ignore
//! // Volatile (in-memory):
//! let store = split::Store::new(
//!     Alloc::new(Global),  // data
//!     Alloc::new(Global),  // index
//! )?;
//!
//! // Non-volatile (file-mapped):
//! let store = split::Store::new(
//!     FileMapped::new(data_file)?,
//!     FileMapped::new(index_file)?,
//! )?;
//! ```
//!
//! Separates data and indexes for better cache efficiency:
//! ```text
//! DataPart:                 IndexPart:
//! +--------+--------+       +----------------+
//! | source | target |       | source_tree    |
//! +--------+--------+       | target_tree    |
//!                           +----------------+
//! ```
//!
//! ## Operations
//!
//! ### Create Point Link
//!
//! Interface method: `store.create_point()`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! let id = self.allocate_next_id();
//! self.links[id] = Link { id, source: id, target: id };
//! self.source_index.insert(id, id);  // index by source
//! self.target_index.insert(id, id);  // index by target
//! ```
//!
//! - Allocates next available ID from internal counter
//! - Writes (id, id, id) tuple directly to memory/file
//! - Updates source and target index trees
//! - Time complexity: O(log n) for index updates
//!
//! ### Update Link
//!
//! Interface method: `store.update(id, source, target)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! let old = self.links[id];
//! self.source_index.remove(old.source, id);
//! self.target_index.remove(old.target, id);
//! self.links[id] = Link { id, source, target };
//! self.source_index.insert(source, id);
//! self.target_index.insert(target, id);
//! ```
//!
//! - Direct memory access to read old values: O(1)
//! - Updates index trees: O(log n)
//! - Writes new values: O(1)
//!
//! ### Delete Link
//!
//! Interface method: `store.delete(id)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! let old = self.links[id];
//! self.source_index.remove(old.source, id);
//! self.target_index.remove(old.target, id);
//! self.links[id] = EMPTY;
//! self.free_list.push(id);  // reuse slot later
//! ```
//!
//! - Direct memory access: O(1)
//! - Index tree updates: O(log n)
//! - Marks slot for reuse
//!
//! ### Query All Links (Each All)
//!
//! Interface method: `store.each(handler)` or `store.each_by([any, any, any], handler)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! for id in 1..=self.count {
//!     if self.links[id].is_valid() {
//!         handler(self.links[id]);
//!     }
//! }
//! ```
//!
//! - Iterates through internal link array sequentially
//! - Skips empty/deleted slots
//! - Time complexity: O(n) where n = total allocated slots
//!
//! ### Query by ID (Each Identity)
//!
//! Interface method: `store.each_by([id, any, any], handler)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! if let Some(link) = self.links.get(id) {
//!     if link.is_valid() {
//!         handler(link);
//!     }
//! }
//! ```
//!
//! - Direct array index access: O(1)
//! - Returns link at `links[id]` if it exists
//! - Fastest possible lookup
//!
//! ### Query by Source (Each Outgoing)
//!
//! Interface method: `store.each_by([any, source, any], handler)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! for id in self.source_index.get_all(source) {
//!     handler(self.links[id]);
//! }
//! ```
//!
//! - Uses source index tree to find all links with given source
//! - Time complexity: O(log n + k) where k = matching links
//! - Finds all outgoing edges from a node
//!
//! ### Query by Target (Each Incoming)
//!
//! Interface method: `store.each_by([any, any, target], handler)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! for id in self.target_index.get_all(target) {
//!     handler(self.links[id]);
//! }
//! ```
//!
//! - Uses target index tree to find all links with given target
//! - Time complexity: O(log n + k) where k = matching links
//! - Finds all incoming edges to a node
//!
//! ### Query by Source AND Target (Each Concrete)
//!
//! Interface method: `store.each_by([any, source, target], handler)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! // Uses whichever index has fewer entries, then filters
//! let candidates = self.source_index.get_all(source);  // or target_index
//! for id in candidates {
//!     if self.links[id].target == target {  // or source check
//!         handler(self.links[id]);
//!     }
//! }
//! ```
//!
//! - Uses one index tree, then filters by the other field
//! - Time complexity: O(log n + k) for tree traversal
//!
//! ### Get Link by ID
//!
//! Interface method: `store.get_link(id)`
//!
//! ```rust,ignore
//! // Implementation (conceptual):
//! self.links.get(id).filter(|l| l.is_valid()).copied()
//! ```
//!
//! - Direct array access: O(1)
//!
//! ## Index Structure
//!
//! Doublets uses balanced trees for source and target indexes:
//!
//! ```text
//! Source Index Tree:
//!       [5]
//!      /   \
//!    [3]   [7]
//!    /       \
//!  [1]       [9]
//!
//! Each node contains: (source_value -> list of link IDs)
//! ```
//!
//! ## Performance Characteristics
//!
//! | Operation       | Time Complexity | Notes                              |
//! |-----------------|-----------------|---------------------------------------|
//! | Create          | O(log n)        | Index tree insertions                 |
//! | Update          | O(log n)        | Index tree remove + insert            |
//! | Delete          | O(log n)        | Index tree removals                   |
//! | Each All        | O(n)            | Full array scan                       |
//! | Each Identity   | O(1)            | Direct array access                   |
//! | Each Outgoing   | O(log n + k)    | Tree lookup + k results               |
//! | Each Incoming   | O(log n + k)    | Tree lookup + k results               |
//! | Each Concrete   | O(log n + k)    | Tree lookup + filter                  |
//!
//! ## Memory Layout
//!
//! ### United Store
//! ```text
//! [Header][Link 1][Link 2][Link 3]...[Link n]
//!    ^       ^
//!    |       +-- Each link: (id, source, target)
//!    +-- Contains: count, free_list_head, etc.
//! ```
//!
//! ### Split Store
//! ```text
//! Data File:
//! [Header][Data 1][Data 2]...[Data n]
//!            ^
//!            +-- Each data: (source, target)
//!
//! Index File:
//! [Header][Source Tree Nodes][Target Tree Nodes]
//! ```

// This is a documentation-only module
