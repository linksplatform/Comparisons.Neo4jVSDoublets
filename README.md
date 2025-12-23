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
| Create        | 619 (59190.8x faster)    | 670 (54685.2x faster)       | 864 (42406.4x faster)   | 854 (42902.9x faster)      | 51789726             | 36639104          |
| Update        | 1426 (26701.8x faster)   | 1424 (26739.3x faster)      | 830 (45875.6x faster)   | 838 (45437.6x faster)      | 38076746             | 38704057          |
| Delete        | 864 (22862.2x faster)    | 863 (22888.7x faster)       | 1457 (13557.3x faster)  | 1465 (13483.2x faster)     | 20386076             | 19752938          |
| Each All      | 71 (12698.9x faster)     | 76 (11863.4x faster)        | 80 (11270.3x faster)    | 77 (11709.4x faster)       | 904778               | 901622            |
| Each Identity | 348 (25691.7x faster)    | 348 (25691.7x faster)       | 348 (25691.7x faster)   | 348 (25691.7x faster)      | 8986314              | 8940727           |
| Each Concrete | 435 (20648.8x faster)    | 435 (20648.8x faster)       | 457 (19654.7x faster)   | 472 (19030.1x faster)      | 8982219              | 8996911           |
| Each Outgoing | 488 (18318.0x faster)    | 487 (18355.6x faster)       | 408 (21909.8x faster)   | 418 (21385.6x faster)      | 8952026              | 8939180           |
| Each Incoming | 494 (17588.2x faster)    | 494 (17588.2x faster)       | 410 (21191.6x faster)   | 411 (21140.1x faster)      | 8689813              | 8688572           |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
