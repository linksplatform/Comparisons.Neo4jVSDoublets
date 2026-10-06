# Comparisons.Neo4jVSDoublets

This repository measures how long basic operations on links take in two
databases, with the same benchmarks written in two languages:

| Language | Neo4j 2026.09 Community Edition (default settings)                     | Doublets (LinksPlatform, in the benchmark process)                                  |
|----------|------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| Rust     | [neo4rs](https://github.com/neo4j-labs/neo4rs) Bolt driver             | [doublets](https://crates.io/crates/doublets) crate                                 |
| C#       | official [Neo4j.Driver](https://www.nuget.org/packages/Neo4j.Driver)   | [Platform.Data.Doublets](https://www.nuget.org/packages/Platform.Data.Doublets) package |

A link (doublet) is a triple `(id, source, target)`. In each language both
databases implement the same interface (the [`Doublets`](https://docs.rs/doublets)
trait in Rust, [`IBenchedLinks`](csharp/Neo4jVSDoublets/IBenchedLinks.cs) in
C#), and every benchmark calls the same methods on all of them. Both languages
send the same Cypher statements and run the same operations with the same
sizes, so their results can be compared with each other.

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
[`rust/src/doublets_impl.rs`](rust/src/doublets_impl.rs), and for C# in
[`csharp/Neo4jVSDoublets/Neo4jLinks.cs`](csharp/Neo4jVSDoublets/Neo4jLinks.cs) and
[`csharp/Neo4jVSDoublets/DoubletsLinks.cs`](csharp/Neo4jVSDoublets/DoubletsLinks.cs).
The C# names of the operations are `CreatePoint`, `Update`, `Delete` and
`Each(id, source, target, visit)`.

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
- The C# benchmarks sample the same way: their
  [harness](csharp/Neo4jVSDoublets/Harness.cs) warms up and chooses the
  number of iterations of every sample with the formulas of Criterion 0.8
  (flat sampling for Neo4j, linear for Doublets), and prints the same
  `bencher` lines (median and standard deviation), so one script reports both
  languages.
- The CI runs every language, database and number of background links in its
  own GitHub Actions job (a separate virtual machine), so they do not compete
  for the same CPU. Neo4j runs in the official `neo4j:2026.09-community`
  Docker image without configuration changes.
- Before the benchmarks, tests check that all seven implementations return the
  same results and that undoing an iteration restores the links
  ([`rust/tests/same_behavior.rs`](rust/tests/same_behavior.rs),
  [`csharp/Neo4jVSDoublets.Tests/SameBehaviorTests.cs`](csharp/Neo4jVSDoublets.Tests/SameBehaviorTests.cs)).

Benchmarks on the main branch use `N = 1,000` and `B` = 10,000, 100,000 and
1,000,000; such a run takes about 12 minutes on GitHub Actions (the longest
job, Neo4j with 1,000,000 background links, about 11 minutes). Pull requests
run a quick check with `N = 100` and `B = 1,000`. Larger `B` are not possible
with the Rust doublets 0.5.0 (see [Limitations](#limitations)). Changes that do not
touch the code, such as this README, start no tests or benchmarks.

## Results

The results below are generated by the CI on every change of the benchmark
code on the main branch. Every cell is the median time of one iteration
(`N` operations), rounded to three significant digits.

- **Bold** marks the fastest Neo4j implementation of each operation (usually
  `Neo4j_Batch`; for Each All, which has no batch variant, the faster of the
  other two).
- Every Doublets cell shows in brackets how many times faster or slower it is
  than this fastest Neo4j implementation: `128× faster` means that Neo4j
  needs 128 times as long for the same work.
- `≈ same` means that the two medians differ by less than 5%, or that their
  ranges of one standard deviation overlap; such a difference is within the
  noise of the measurement.
- The line above every table names the versions, the CPU and the GitHub
  Actions run that measured it.
- In the linear charts, bars shorter than 0.5% of the longest bar are drawn
  0.5% long to stay visible; the log charts show every bar to scale.

<!-- results:start -->
<!-- markdownlint-disable MD013 MD024 -->

### Rust

#### Rust: 10,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through neo4rs 0.8.0; doublets 0.5.0. CPU AMD EPYC 9V74 80-Core Processor (Neo4j) and AMD EPYC 7763 64-Core Processor (Doublets), [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 79.8 µs (120× faster) | 79.7 µs (121× faster) | 51.5 µs (187× faster) | 51.8 µs (186× faster) | 869 ms | 393 ms | **9.62 ms** |
| Update | 396 µs (162× faster) | 390 µs (164× faster) | 39.1 µs (1,640× faster) | 39.1 µs (1,640× faster) | 1.5 s | 795 ms | **64.1 ms** |
| Delete | 180 µs (47.1× faster) | 180 µs (47.2× faster) | 105 µs (80.6× faster) | 104 µs (81.4× faster) | 758 ms | 391 ms | **8.49 ms** |
| Each All | 13.1 µs (2,610× faster) | 13.4 µs (2,540× faster) | 69.7 µs (489× faster) | 69.1 µs (493× faster) | **34.1 ms** | 34.5 ms | — |
| Each Identity | 5.01 µs (1,290× faster) | 5.01 µs (1,290× faster) | 9.7 µs (669× faster) | 9.7 µs (669× faster) | 581 ms | 378 ms | **6.48 ms** |
| Each Concrete | 34.6 µs (188× faster) | 34.3 µs (189× faster) | 16.5 µs (393× faster) | 16.9 µs (385× faster) | 585 ms | 386 ms | **6.5 ms** |
| Each Outgoing | 28.3 µs (227× faster) | 28.5 µs (225× faster) | 14.1 µs (457× faster) | 14.1 µs (456× faster) | 579 ms | 384 ms | **6.42 ms** |
| Each Incoming | 63.9 µs (101× faster) | 66.1 µs (97.5× faster) | 16.5 µs (390× faster) | 16.5 µs (390× faster) | 576 ms | 389 ms | **6.44 ms** |

![Rust, 10,000 background links, 1,000 links per iteration, linear scale](Docs/bench_rust_10000.png)
![Rust, 10,000 background links, 1,000 links per iteration, log scale](Docs/bench_rust_log_scale_10000.png)

#### Rust: 100,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through neo4rs 0.8.0; doublets 0.5.0. CPU INTEL(R) XEON(R) PLATINUM 8573C (Neo4j) and AMD EPYC 9V74 80-Core Processor (Doublets), [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 80.8 µs (116× faster) | 80.8 µs (116× faster) | 37.3 µs (251× faster) | 37.3 µs (251× faster) | 911 ms | 307 ms | **9.38 ms** |
| Update | 365 µs (184× faster) | 363 µs (185× faster) | 28.5 µs (2,350× faster) | 28.4 µs (2,360× faster) | 1.64 s | 626 ms | **67.1 ms** |
| Delete | 146 µs (62.9× faster) | 149 µs (61.9× faster) | 81.3 µs (113× faster) | 80.6 µs (114× faster) | 822 ms | 312 ms | **9.19 ms** |
| Each All | 108 µs (2,930× faster) | 110 µs (2,870× faster) | 630 µs (501× faster) | 630 µs (502× faster) | **316 ms** | 322 ms | — |
| Each Identity | 3.88 µs (1,620× faster) | 3.88 µs (1,620× faster) | 8.49 µs (740× faster) | 8.49 µs (740× faster) | 461 ms | 290 ms | **6.28 ms** |
| Each Concrete | 37.1 µs (172× faster) | 37.2 µs (172× faster) | 14.3 µs (448× faster) | 14.3 µs (448× faster) | 466 ms | 294 ms | **6.39 ms** |
| Each Outgoing | 30.4 µs (204× faster) | 30.5 µs (204× faster) | 11.5 µs (538× faster) | 11.5 µs (538× faster) | 469 ms | 303 ms | **6.21 ms** |
| Each Incoming | 51.2 µs (122× faster) | 51.6 µs (121× faster) | 11.8 µs (530× faster) | 11.8 µs (530× faster) | 454 ms | 293 ms | **6.24 ms** |

![Rust, 100,000 background links, 1,000 links per iteration, linear scale](Docs/bench_rust_100000.png)
![Rust, 100,000 background links, 1,000 links per iteration, log scale](Docs/bench_rust_log_scale_100000.png)

#### Rust: 1,000,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through neo4rs 0.8.0; doublets 0.5.0. CPU AMD EPYC 7763 64-Core Processor, [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 113 µs (87.6× faster) | 113 µs (88× faster) | 51.1 µs (194× faster) | 51.5 µs (193× faster) | 978 ms | 476 ms | **9.94 ms** |
| Update | 520 µs (129× faster) | 511 µs (132× faster) | 39.2 µs (1,710× faster) | 39.2 µs (1,720× faster) | 1.71 s | 958 ms | **67.2 ms** |
| Delete | 250 µs (37.8× faster) | 248 µs (38.1× faster) | 105 µs (89.5× faster) | 106 µs (89.3× faster) | 922 ms | 480 ms | **9.44 ms** |
| Each All | 2.67 ms (1,360× faster) | 2.97 ms (1,220× faster) | 7.05 ms (513× faster) | 7.12 ms (508× faster) | **3.62 s** | 3.65 s | — |
| Each Identity | 5.01 µs (1,470× faster) | 5.01 µs (1,470× faster) | 9.7 µs (760× faster) | 9.69 µs (761× faster) | 707 ms | 468 ms | **7.37 ms** |
| Each Concrete | 53.9 µs (130× faster) | 59.2 µs (118× faster) | 16.5 µs (424× faster) | 17.2 µs (408× faster) | 707 ms | 470 ms | **7.01 ms** |
| Each Outgoing | 59.9 µs (116× faster) | 60.2 µs (115× faster) | 14.1 µs (492× faster) | 14.1 µs (492× faster) | 694 ms | 466 ms | **6.92 ms** |
| Each Incoming | 107 µs (65.2× faster) | 105 µs (66.2× faster) | 16.5 µs (421× faster) | 16.5 µs (421× faster) | 693 ms | 464 ms | **6.96 ms** |

![Rust, 1,000,000 background links, 1,000 links per iteration, linear scale](Docs/bench_rust_1000000.png)
![Rust, 1,000,000 background links, 1,000 links per iteration, log scale](Docs/bench_rust_log_scale_1000000.png)

### C#

#### C#: 10,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through Neo4j.Driver 6.3.0; Platform.Data.Doublets 0.18.1. CPU AMD EPYC 9V74 80-Core Processor (Neo4j) and AMD EPYC 7763 64-Core Processor (Doublets), [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 660 µs (14× faster) | 638 µs (14.5× faster) | 195 µs (47.4× faster) | 180 µs (51.2× faster) | 1.24 s | 334 ms | **9.23 ms** |
| Update | 1.02 ms (20.5× faster) | 1.01 ms (20.7× faster) | 126 µs (166× faster) | 126 µs (166× faster) | 1.79 s | 714 ms | **20.9 ms** |
| Delete | 288 µs (31.7× faster) | 289 µs (31.7× faster) | 154 µs (59.4× faster) | 164 µs (55.8× faster) | 1.31 s | 354 ms | **9.14 ms** |
| Each All | 190 µs (106× faster) | 191 µs (106× faster) | 169 µs (119× faster) | 170 µs (119× faster) | **20.2 ms** | 20.7 ms | — |
| Each Identity | 36.5 µs (118× faster) | 36.4 µs (118× faster) | 56.5 µs (76.1× faster) | 56.4 µs (76.3× faster) | 524 ms | 335 ms | **4.3 ms** |
| Each Concrete | 67.1 µs (65.1× faster) | 73.8 µs (59.2× faster) | 60.4 µs (72.3× faster) | 60.4 µs (72.3× faster) | 1.45 s | 334 ms | **4.37 ms** |
| Each Outgoing | 101 µs (42.2× faster) | 98.8 µs (43× faster) | 60.6 µs (70.1× faster) | 59.8 µs (71× faster) | 522 ms | 333 ms | **4.25 ms** |
| Each Incoming | 154 µs (27.4× faster) | 152 µs (27.6× faster) | 62.2 µs (67.7× faster) | 61.2 µs (68.8× faster) | 521 ms | 336 ms | **4.21 ms** |

![C#, 10,000 background links, 1,000 links per iteration, linear scale](Docs/bench_csharp_10000.png)
![C#, 10,000 background links, 1,000 links per iteration, log scale](Docs/bench_csharp_log_scale_10000.png)

#### C#: 100,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through Neo4j.Driver 6.3.0; Platform.Data.Doublets 0.18.1. CPU AMD EPYC 7763 64-Core Processor (Neo4j) and AMD EPYC 9V45 96-Core Processor (Doublets), [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 419 µs (24.2× faster) | 411 µs (24.7× faster) | 75.5 µs (134× faster) | 84.4 µs (120× faster) | 1.12 s | 472 ms | **10.1 ms** |
| Update | 628 µs (41.4× faster) | 631 µs (41.2× faster) | 74.2 µs (351× faster) | 75.2 µs (346× faster) | 1.97 s | 1.02 s | **26 ms** |
| Delete | 219 µs (46.3× faster) | 224 µs (45.3× faster) | 82.6 µs (122× faster) | 91.3 µs (111× faster) | 1.01 s | 504 ms | **10.1 ms** |
| Each All | 1.36 ms (188× faster) | 1.33 ms (191× faster) | 1.12 ms (228× faster) | 1.09 ms (233× faster) | **255 ms** | 263 ms | — |
| Each Identity | 23.8 µs (248× faster) | 24.1 µs (245× faster) | 35.9 µs (164× faster) | 35.3 µs (167× faster) | 790 ms | 481 ms | **5.9 ms** |
| Each Concrete | 76 µs (74.8× faster) | 83.2 µs (68.3× faster) | 37.6 µs (151× faster) | 38 µs (150× faster) | 790 ms | 482 ms | **5.69 ms** |
| Each Outgoing | 87.8 µs (63.8× faster) | 84.6 µs (66.2× faster) | 37.9 µs (148× faster) | 38.3 µs (146× faster) | 776 ms | 479 ms | **5.6 ms** |
| Each Incoming | 140 µs (40.6× faster) | 130 µs (43.6× faster) | 38.1 µs (149× faster) | 38.6 µs (146× faster) | 771 ms | 476 ms | **5.66 ms** |

![C#, 100,000 background links, 1,000 links per iteration, linear scale](Docs/bench_csharp_100000.png)
![C#, 100,000 background links, 1,000 links per iteration, log scale](Docs/bench_csharp_log_scale_100000.png)

#### C#: 1,000,000 background links, 1,000 links per iteration

_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through Neo4j.Driver 6.3.0; Platform.Data.Doublets 0.18.1. CPU Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz (Neo4j) and AMD EPYC 7763 64-Core Processor (Doublets), [GitHub Actions run](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/37524891785) on 2026-10-06._

| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Create | 913 µs (11.6× faster) | 935 µs (11.3× faster) | 177 µs (60× faster) | 195 µs (54.3× faster) | 967 ms | 368 ms | **10.6 ms** |
| Update | 1.32 ms (23.2× faster) | 1.32 ms (23.3× faster) | 130 µs (236× faster) | 128 µs (240× faster) | 1.73 s | 830 ms | **30.7 ms** |
| Delete | 462 µs (21.8× faster) | 470 µs (21.4× faster) | 165 µs (61.2× faster) | 190 µs (53.2× faster) | 841 ms | 390 ms | **10.1 ms** |
| Each All | 22.4 ms (124× faster) | 22 ms (126× faster) | 18.4 ms (150× faster) | 19.3 ms (144× faster) | 2.8 s | **2.77 s** | — |
| Each Identity | 44.7 µs (147× faster) | 41.9 µs (157× faster) | 61.8 µs (106× faster) | 64.1 µs (103× faster) | 604 ms | 365 ms | **6.57 ms** |
| Each Concrete | 142 µs (46.2× faster) | 145 µs (45.2× faster) | 68.9 µs (95.5× faster) | 69.8 µs (94.2× faster) | 618 ms | 369 ms | **6.58 ms** |
| Each Outgoing | 167 µs (36.9× faster) | 162 µs (38.1× faster) | 70.7 µs (87.3× faster) | 70.4 µs (87.6× faster) | 602 ms | 369 ms | **6.17 ms** |
| Each Incoming | 208 µs (29.7× faster) | 217 µs (28.6× faster) | 71.2 µs (87× faster) | 71.7 µs (86.4× faster) | 584 ms | 380 ms | **6.19 ms** |

![C#, 1,000,000 background links, 1,000 links per iteration, linear scale](Docs/bench_csharp_1000000.png)
![C#, 1,000,000 background links, 1,000 links per iteration, log scale](Docs/bench_csharp_log_scale_1000000.png)

<!-- markdownlint-restore -->
<!-- results:end -->

## Limitations

- **Driver round trips (Rust).** neo4rs needs 3 network round trips for a statement
  outside of an explicit transaction and 2 inside one. The
  [`neo4j`](https://crates.io/crates/neo4j) crate, which follows the API of the
  official drivers, needs 2 and 1 because it sends several Bolt messages at
  once ([`experiments`](experiments)). Neo4j does not publish an official Rust
  driver; the C# benchmarks use the official .NET driver.
- **One client.** The benchmarks run operations one after another from one
  thread. Throughput with concurrent clients is not measured.
- **Only time is measured.** Memory use and disk I/O are not collected.
- **Each Concrete** returns at most one link in Doublets, which treats a
  `(source, target)` pair as unique; the benchmarks never create two links
  with the same pair.
- **At most 1,040,383 links (Rust).** The doublets 0.5.0 stores panic when link
  number 1,040,384 is created: after growing their memory, they use only the
  newly added part of it as the whole array (see
  [`experiments/store-capacity`](experiments/store-capacity) and the
  "Capacity" section of [`rust/src/doublets_impl.rs`](rust/src/doublets_impl.rs)).
  This is why `B` stops at 1,000,000. The C# Platform.Data.Doublets stores
  have no such limit, but use the same sizes, so the results of both languages
  stay comparable.
- **Point links only.** The Rust doublets 0.5.0 Split store does not find links
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
docker run -d --name neo4j -p 7687:7687 -p 7474:7474 -e NEO4J_AUTH=neo4j/password neo4j:2026.09-community

cd rust
cargo test --release -- --include-ignored     # check that all stores agree
cargo bench --bench bench                     # both databases
BENCHMARK_BACKEND=doublets cargo bench --bench bench   # Doublets only, no server needed

cd ../csharp                                  # .NET SDK 10
NEO4J_URI=bolt://localhost:7687 dotnet test   # check that all stores agree (without NEO4J_URI the Neo4j tests are skipped)
dotnet run -c Release --project Neo4jVSDoublets          # both databases
BENCHMARK_BACKEND=doublets dotnet run -c Release --project Neo4jVSDoublets   # Doublets only
dotnet run -c Release --project Neo4jVSDoublets Each_    # only benchmarks whose name contains Each_
```

Both languages read the same environment variables:

| Environment variable         | Default                 | Meaning                                    |
|------------------------------|-------------------------|--------------------------------------------|
| `BENCHMARK_LINKS`            | `100`                   | `N`, operations per iteration              |
| `BENCHMARK_BACKGROUND_LINKS` | `1000`                  | `B`, links that exist before an iteration  |
| `BENCHMARK_BACKEND`          | both                    | `neo4j` or `doublets`                      |
| `NEO4J_URI`                  | `bolt://localhost:7687` | Neo4j server                               |
| `NEO4J_USER`                 | `neo4j`                 |                                            |
| `NEO4J_PASSWORD`             | `password`              |                                            |

The tables and charts are generated with
[`scripts/benchmark_report.py`](scripts/benchmark_report.py) from the output of
`cargo bench --bench bench -- --output-format bencher` (Rust) or
`dotnet run -c Release --project Neo4jVSDoublets` (C#), saved as
`results/<rust|csharp>-<B>-<neo4j|doublets>.txt` after the lines that
[`scripts/benchmark_header.sh`](scripts/benchmark_header.sh) prints:

```bash
mkdir -p results
export BENCHMARK_LINKS=100 BENCHMARK_BACKGROUND_LINKS=1000
for backend in neo4j doublets; do
  { scripts/benchmark_header.sh $backend
    (cd rust && BENCHMARK_BACKEND=$backend cargo bench --bench bench -- --output-format bencher --noplot)
  } > results/rust-1000-$backend.txt
  { scripts/benchmark_header.sh $backend
    (cd csharp && BENCHMARK_BACKEND=$backend dotnet run -c Release --project Neo4jVSDoublets)
  } > results/csharp-1000-$backend.txt
done
pip install matplotlib
python3 scripts/benchmark_report.py results --readme README.md --charts Docs
python3 -m unittest discover -s scripts       # tests of the report
```
