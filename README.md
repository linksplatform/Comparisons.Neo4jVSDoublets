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
| Create        | 611 (61154.8x faster)    | 627 (59594.2x faster)       | 834 (44802.8x faster)   | 826 (45236.8x faster)      | 67353721             | 37365570          |
| Update        | 1435 (26744.7x faster)   | 1439 (26670.4x faster)      | 832 (46128.2x faster)   | 835 (45962.5x faster)      | 38378690             | 38625040          |
| Delete        | 864 (22821.8x faster)    | 858 (22981.4x faster)       | 1460 (13505.5x faster)  | 1500 (13145.4x faster)     | 20491228             | 19718029          |
| Each All      | 71 (12619.6x faster)     | 75 (11946.6x faster)        | 78 (11487.1x faster)    | 75 (11946.6x faster)       | 911366               | 895992            |
| Each Identity | 348 (25839.9x faster)    | 348 (25839.9x faster)       | 348 (25839.9x faster)   | 348 (25839.9x faster)      | 9057924              | 8992288           |
| Each Concrete | 438 (20347.9x faster)    | 436 (20441.2x faster)       | 455 (19587.6x faster)   | 454 (19630.8x faster)      | 8979834              | 8912371           |
| Each Outgoing | 489 (18286.3x faster)    | 488 (18323.8x faster)       | 407 (21970.5x faster)   | 407 (21970.5x faster)      | 8941998              | 8987035           |
| Each Incoming | 495 (18017.9x faster)    | 494 (18054.4x faster)       | 411 (21700.4x faster)   | 411 (21700.4x faster)      | 8918865              | 8934565           |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
