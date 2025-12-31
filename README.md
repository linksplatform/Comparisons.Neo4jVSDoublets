# Comparisons.Neo4jVSDoublets

The comparison between Neo4j and LinksPlatform's Doublets (links) on basic database operations with links (create, read, delete, update).
All benchmarks ran with 3000 links in background to increase size of indexes and 1000 are actively created/updated/deleted.

In this particular benchmark we decided not to increase the number of links as Neo4j will not be able to handle it at all in timeframe what GitHub Actions limit allows to use for free. Remember that to get accurate result we ran this benchmark multiple times.

## Task

Both databases used to store and retrieve doublet-links representation. To support storage, and all basic CRUD operations that provide Turing completeness for links as in [the links theory](https://habr.com/ru/articles/895896).

## Operations
- **Create** – insert point link (link with id = source = target)
- **Update** – basic link update operation
- **Delete** – basic link delete operation
- **Each All** – take all links matching `[*, *, *]` constraint
- **Each Incoming** – take all links matching `[*, *, target]` constraint
- **Each Outgoing** – take all links matching `[*, source, *]` constraint
- **Each Concrete** – take all links matching `[*, source, target]` constraint
- **Each Identity** – take all links matching `[id, *, *]` constraint

## Results
The results below represent the amount of time (ns) the operation takes per iteration.
- First picture shows time in a pixel scale (for doublets just minimum value is shown, otherwise it will be not present on the graph).
- Second picture shows time in a logarithmic scale (to see difference clearly, because it is around 2-3 orders of magnitude).

### Rust
![Image of Rust benchmark (pixel scale)](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/blob/main/Docs/bench_rust.png?raw=true)
![Image of Rust benchmark (log scale)](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/blob/main/Docs/bench_rust_log_scale.png?raw=true)

### Raw benchmark results (all numbers are in nanoseconds)

| Operation     | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction |
|---------------|--------------------------|-----------------------------|-------------------------|----------------------------|----------------------|-------------------|
| Create        | 84823 (36630.1x faster)  | 86001 (36128.3x faster)     | 84120 (36936.2x faster) | 83188 (37350.0x faster)    | 3159452615           | 3107071983        |
| Update        | 1484 (26276.2x faster)   | 1539 (25337.2x faster)      | 833 (46811.4x faster)   | 836 (46643.4x faster)      | 38993916             | 39342594          |
| Delete        | 143527 (13702.8x faster) | 153180 (12839.3x faster)    | 145585 (13509.1x faster) | 143649 (13691.2x faster)   | 1989558566           | 1966720201        |
| Each All      | 74 (12500.5x faster)     | 77 (12013.5x faster)        | 78 (11859.5x faster)    | 77 (12013.5x faster)       | 943914               | 925040            |
| Each Identity | 348 (25170.8x faster)    | 348 (25170.8x faster)       | 348 (25170.8x faster)   | 348 (25170.8x faster)      | 8801871              | 8759426           |
| Each Concrete | 439 (20001.8x faster)    | 438 (20047.5x faster)       | 470 (18682.6x faster)   | 451 (19469.6x faster)      | 8816664              | 8780803           |
| Each Outgoing | 677 (12894.7x faster)    | 493 (17707.3x faster)       | 422 (20686.5x faster)   | 408 (21396.3x faster)      | 8729692              | 8753819           |
| Each Incoming | 671 (13028.9x faster)    | 487 (17951.5x faster)       | 411 (21271.0x faster)   | 411 (21271.0x faster)      | 8742366              | 8792452           |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
