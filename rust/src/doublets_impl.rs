//! # Doublets Implementation
//!
//! This module documents how the [doublets](https://crates.io/crates/doublets)
//! stores implement the [`Doublets`](doublets::Doublets) interface. The stores
//! run inside the benchmark process: every operation reads and writes the
//! store's memory directly, without a server, a network connection or a query
//! language.
//!
//! ## Stores
//!
//! A link is identified by its position in an array: link `id` is element
//! `id` of the array. Two size-balanced trees index the links, one ordered by
//! `(source, target)` and one ordered by `(target, source)`. The tree nodes
//! are stored in the array elements as well, so no memory is allocated per
//! link.
//!
//! ### United (unit) store
//!
//! One array; each element holds the link and its nodes of both trees:
//!
//! ```text
//! LinkPart: source, target,
//!           left_as_source, right_as_source, size_as_source,
//!           left_as_target, right_as_target, size_as_target
//! ```
//!
//! ```
//! use doublets::{mem::Global, unit, Doublets};
//!
//! let mut store = unit::Store::<usize, _>::new(Global::new())?;
//! let id = store.create_point()?;
//! assert_eq!(store.get_link(id).map(|link| (link.source, link.target)), Some((id, id)));
//! # Ok::<(), doublets::Error<usize>>(())
//! ```
//!
//! ### Split store
//!
//! Two arrays: the link data and the tree nodes are kept apart, so reading
//! links does not load tree nodes into the CPU cache:
//!
//! ```text
//! DataPart:  source, target
//! IndexPart: root_as_source, left_as_source, right_as_source, size_as_source,
//!            root_as_target, left_as_target, right_as_target, size_as_target
//! ```
//!
//! ```
//! use doublets::{mem::Global, split, Doublets};
//!
//! let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
//! let id = store.create_point()?;
//! assert_eq!(store.get_link(id).map(|link| (link.source, link.target)), Some((id, id)));
//! # Ok::<(), doublets::Error<usize>>(())
//! ```
//!
//! ### Volatile and non-volatile
//!
//! The arrays are kept either in RAM (`Global`, "Volatile") or in
//! memory-mapped files (`FileMapped`, "NonVolatile", see
//! [`map_file`](crate::map_file)). Writes to a memory-mapped file go to the OS
//! page cache; the benchmarks do not call `fsync`.
//!
//! ## Operations
//!
//! `n` is the number of links, `k` the number of found links.
//!
//! | Operation       | Method                       | Implementation                                                        | Time         |
//! |-----------------|------------------------------|-----------------------------------------------------------------------|--------------|
//! | Create          | `create_point()`             | take a free element (a deleted one or the next one), write `(id, id)`, insert into both trees | O(log n) |
//! | Update          | `update(id, source, target)` | remove from both trees, write the new values, insert into both trees  | O(log n)     |
//! | Delete          | `delete(id)`                 | update to `(0, 0)`, then mark the element as free                     | O(log n)     |
//! | Each All        | `each([*, *, *])`            | scan the array, skipping free elements                                | O(n)         |
//! | Each Identity   | `each_by([id, *, *])`        | read element `id`                                                     | O(1)         |
//! | Each Concrete   | `each_by([*, source, target])` | search the `(source, target)` tree                                  | O(log n)     |
//! | Each Outgoing   | `each_by([*, source, *])`    | walk the `(source, target)` tree from the first link with `source`    | O(log n + k) |
//! | Each Incoming   | `each_by([*, *, target])`    | walk the `(target, source)` tree from the first link with `target`    | O(log n + k) |
//!
//! Links with `source = target = 0` are not inserted into the trees.
//!
//! A concrete query returns at most one link, because doublets treats a
//! `(source, target)` pair as unique. The benchmarks never create two links
//! with the same pair.

// This is a documentation-only module.
