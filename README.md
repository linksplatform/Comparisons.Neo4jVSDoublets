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
| Create        | 611 (57765.2x faster)    | 620 (56926.6x faster)       | 834 (42319.6x faster)   | 858 (41135.8x faster)      | 49120269             | 35294507          |
| Update        | 1414 (26913.2x faster)   | 1416 (26875.2x faster)      | 834 (45629.8x faster)   | 843 (45142.6x faster)      | 38055226             | 38236408          |
| Delete        | 863 (22702.9x faster)    | 860 (22782.1x faster)       | 1450 (13512.2x faster)  | 1538 (12739.0x faster)     | 20388133             | 19592619          |
| Each All      | 70 (12643.3x faster)     | 75 (11800.5x faster)        | 78 (11346.6x faster)    | 91 (9725.6x faster)        | 890548               | 885034            |
| Each Identity | 348 (25015.2x faster)    | 348 (25015.2x faster)       | 347 (25087.3x faster)   | 355 (24522.0x faster)      | 8717748              | 8705304           |
| Each Concrete | 439 (19801.3x faster)    | 437 (19891.9x faster)       | 459 (18938.5x faster)   | 488 (17813.0x faster)      | 8699769              | 8692766           |
| Each Outgoing | 489 (17696.7x faster)    | 489 (17696.7x faster)       | 408 (21210.0x faster)   | 426 (20313.8x faster)      | 8668635              | 8653675           |
| Each Incoming | 495 (17403.3x faster)    | 494 (17438.5x faster)       | 411 (20960.2x faster)   | 442 (19490.1x faster)      | 8614639              | 8619381           |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
