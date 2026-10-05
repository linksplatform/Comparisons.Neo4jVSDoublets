# Comparisons.Neo4jVSDoublets

This repository measures how long basic operations on links take in two
databases:

- **Neo4j** 2026.09 Community Edition with default settings, used from Rust
  through the [neo4rs](https://github.com/neo4j-labs/neo4rs) Bolt driver;
- **Doublets**, the [doublets](https://crates.io/crates/doublets) Rust
  library from LinksPlatform, used in the same process as the benchmark.

A link (doublet) is a triple `(id, source, target)`. Both databases implement
the same [`Doublets`](https://docs.rs/doublets) interface, and every benchmark
calls the same methods on all of them.

The measured value is the time until an operation is finished from the point
of view of the caller. Two workloads are reported:

- **One operation per call**: every link operation is a separate call through
  the `Doublets` interface, as an application calls it one at a time.
- **Batch** (Neo4j only): the same operations of an iteration are sent to
  Neo4j as one list in one Cypher statement, so the fixed cost of a statement
  (round trips, query planning, commit) is paid once per iteration instead of
  once per operation. Doublets has no batch calls; its batch work is the same
  sequence of function calls as in the first workload.

## What is compared

| Implementation                | Where the data is                          | A finished operation is                            |
|-------------------------------|--------------------------------------------|----------------------------------------------------|
| `Doublets_United_Volatile`    | RAM of the benchmark process               | in RAM                                             |
| `Doublets_United_NonVolatile` | memory-mapped file                         | in the OS page cache (no `fsync`)                  |
| `Doublets_Split_Volatile`     | RAM of the benchmark process               | in RAM                                             |
| `Doublets_Split_NonVolatile`  | two memory-mapped files                    | in the OS page cache (no `fsync`)                  |
| `Neo4j_NonTransaction`        | Neo4j server, reached over Bolt (TCP)      | committed: every statement is its own transaction  |
| `Neo4j_Transaction`           | Neo4j server, reached over Bolt (TCP)      | committed: one explicit transaction per iteration  |
| `Neo4j_Batch`                 | Neo4j server, reached over Bolt (TCP)      | committed: one statement with all `N` operations   |

- **United** and **Split** are two Doublets storage layouts: link data and
  index trees in one array, or in two separate arrays.
- Neo4j can only be reached over the network, so every Neo4j operation
  includes at least one round trip to the server. Doublets has no server: its
  operations are function calls on the process memory. The comparison
  therefore shows the cost of each way of storing links as it is used, not the
  cost of the storage engines alone.
- Neo4j writes every commit to its transaction log on disk. The Doublets stores
  do not call `fsync`.

## Operations

| Operation     | `Doublets` method            | Neo4j (Cypher, one statement)                              | Doublets                                     |
|---------------|------------------------------|------------------------------------------------------------|----------------------------------------------|
| Create        | `create_point()`             | `CREATE (:Link {id: $id, source: $id, target: $id})`       | write the link, insert it into both trees    |
| Update        | `update(id, source, target)` | `MATCH (l:Link {id: $id}) ... SET l.source = ..., l.target = ...` | remove from both trees, write, insert  |
| Delete        | `delete(id)`                 | `MATCH (l:Link {id: $id}) ... DELETE l`                    | remove from both trees, mark as free         |
| Each All      | `each([*, *, *])`            | `MATCH (l:Link) RETURN ...`                                | scan the array                               |
| Each Identity | `each_by([id, *, *])`        | `MATCH (l:Link) WHERE l.id = $id RETURN ...`               | read array element `id`                      |
| Each Concrete | `each_by([*, source, target])` | `MATCH (l:Link) WHERE l.source = $source AND l.target = $target RETURN ...` | search the `(source, target)` tree |
| Each Outgoing | `each_by([*, source, *])`    | `MATCH (l:Link) WHERE l.source = $source RETURN ...`       | walk the `(source, target)` tree             |
| Each Incoming | `each_by([*, *, target])`    | `MATCH (l:Link) WHERE l.target = $target RETURN ...`       | walk the `(target, source)` tree             |

In Neo4j a link is a node `(:Link {id, source, target})`. A link is found by
its `id` property, which has a uniqueness constraint and therefore an index;
`source` and `target` have indexes too. The internal node id (`elementId`) is
not used as the link id, because Neo4j chooses it itself and may reuse it,
while link ids are chosen by the caller. Update and Delete are single
statements that also return the previous `source` and `target`, which the
`Doublets` interface reports.

`Neo4j_Batch` sends the `N` operations of an iteration as a list parameter
and runs them with `UNWIND`, for example
`UNWIND $ids AS id MATCH (l:Link {id: id}) DELETE l`. Neo4j finds each link of
the list with the same index seek as the single statement. Update is two
statements (to `(0, 0)` and back), every other operation one. Each All is
already one statement, so it has no batch variant.

The exact statements and data structures are documented in
[`rust/src/neo4j_impl.rs`](rust/src/neo4j_impl.rs) and
[`rust/src/doublets_impl.rs`](rust/src/doublets_impl.rs).

## Method

- `B` (background links) point links `1..=B` exist before every iteration, so
  the indexes have a realistic size.
- One iteration performs `N` (links per iteration) operations:

  | Benchmark     | One iteration                                                  |
  |---------------|----------------------------------------------------------------|
  | Create        | creates `N` links (ids `B + 1..=B + N`)                        |
  | Update        | `N` × (`update(id, 0, 0)` and `update(id, id, id)`) = `2N` updates |
  | Delete        | deletes links `B` down to `B - N + 1`                          |
  | Each All      | one query that returns all `B` links                           |
  | Each Identity, Each Concrete, Each Outgoing, Each Incoming | `N` queries, for ids `1..=N`, each returning one link |

- The measured time of an iteration starts before the transaction begins and
  ends after it is committed (for `Neo4j_Transaction`). After the measurement
  the changes of the iteration are undone (Create is undone by deleting the
  created links, Delete by creating them again), so every iteration starts
  from the same links. The undo is not measured.
- The background links are created once per benchmark and are not measured.
- [Criterion.rs](https://github.com/bheisler/criterion.rs) collects the
  samples: 10 samples (at least 5 s) for Neo4j, 100 samples for Doublets. The
  tables show the median time of one iteration.
- The CI runs the Neo4j and Doublets benchmarks in separate GitHub Actions jobs
  (separate virtual machines), and each number of background links in its own
  job, so they do not compete for the same CPU. Neo4j runs in the official
  `neo4j:2026.09-community` Docker image without configuration changes.
- Before the benchmarks, a test checks that all six implementations return the
  same results and that undoing an iteration restores the links
  ([`rust/tests/same_behavior.rs`](rust/tests/same_behavior.rs)).

Benchmarks on the main branch use `N = 1,000` and `B` = 10,000, 100,000 and
1,000,000; such a run takes about 12 minutes on GitHub Actions (the longest
job, Neo4j with 1,000,000 background links, about 11 minutes). Pull requests
run a quick check with `N = 100` and `B = 1,000`. Larger `B` are not possible
with doublets 0.5.0 (see [Limitations](#limitations)). Changes that do not
touch the code, such as this README, start no tests or benchmarks.

## Results

The results below are generated by the CI on every change of the benchmark
code on the main branch.

<!-- results:start -->
The results will appear here after the benchmarks run on the main branch.
<!-- results:end -->

## Limitations

- **Driver round trips.** neo4rs needs 3 network round trips for a statement
  outside of an explicit transaction and 2 inside one. The
  [`neo4j`](https://crates.io/crates/neo4j) crate, which follows the API of the
  official drivers, needs 2 and 1 because it sends several Bolt messages at
  once ([`experiments`](experiments)). Neo4j does not publish an official Rust
  driver.
- **One client.** The benchmarks run operations one after another from one
  thread. Throughput with concurrent clients is not measured.
- **Only time is measured.** Memory use and disk I/O are not collected.
- **Each Concrete** returns at most one link in Doublets, which treats a
  `(source, target)` pair as unique; the benchmarks never create two links
  with the same pair.
- **At most 1,040,383 links.** The doublets 0.5.0 stores panic when link
  number 1,040,384 is created: after growing their memory, they use only the
  newly added part of it as the whole array (see
  [`experiments/store-capacity`](experiments/store-capacity) and the
  "Capacity" section of [`rust/src/doublets_impl.rs`](rust/src/doublets_impl.rs)).
  This is why `B` stops at 1,000,000.
- **Point links only.** The doublets 0.5.0 Split store does not find links
  with `source != target` by `[*, source, *]` and `[*, source, target]` queries
  after an `update` (see
  [`experiments/split-store-bug`](experiments/split-store-bug)). The
  benchmarks only use point links and links updated to `(0, 0)` and back, for
  which all stores return the same results.
- Results before [issue #15](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/issues/15)
  were measured with an HTTP client that opened a new connection for every
  request and are not comparable with the current results.

## Running locally

```bash
docker run -d --name neo4j -p 7687:7687 -e NEO4J_AUTH=neo4j/password neo4j:2026.09-community

cd rust
cargo test --release -- --include-ignored     # check that all stores agree
cargo bench --bench bench                     # both databases
BENCHMARK_BACKEND=doublets cargo bench --bench bench   # Doublets only, no server needed
```

| Environment variable         | Default                 | Meaning                                    |
|------------------------------|-------------------------|--------------------------------------------|
| `BENCHMARK_LINKS`            | `100`                   | `N`, operations per iteration              |
| `BENCHMARK_BACKGROUND_LINKS` | `1000`                  | `B`, links that exist before an iteration  |
| `BENCHMARK_BACKEND`          | both                    | `neo4j` or `doublets`                      |
| `NEO4J_URI`                  | `bolt://localhost:7687` | Neo4j server                               |
| `NEO4J_USER`                 | `neo4j`                 |                                            |
| `NEO4J_PASSWORD`             | `password`              |                                            |

The tables and charts are generated with `rust/out.py` from the output of
`cargo bench --bench bench -- --output-format bencher`.
