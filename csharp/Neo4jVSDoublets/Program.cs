// Usage: neo4j-vs-doublets [FILTER]
//
// Runs the benchmarks of both databases, or only of the one named by the `BENCHMARK_BACKEND`
// environment variable (`neo4j` or `doublets`), and prints one line per benchmark in the format
// of `cargo bench -- --output-format bencher`. Only benchmarks whose `Group/Implementation` name
// contains FILTER run. The sizes come from `BENCHMARK_BACKGROUND_LINKS` and `BENCHMARK_LINKS`,
// the Neo4j server from `NEO4J_URI`, `NEO4J_USER` and `NEO4J_PASSWORD`.

using Comparisons.Neo4jVSDoublets;

var filter = args.Length > 0 ? args[0] : "";
var backend = Settings.Backend;
if (backend is not ("" or "neo4j" or "doublets"))
{
    throw new ArgumentException($"BENCHMARK_BACKEND must be `neo4j` or `doublets`, got `{backend}`");
}
var (background, links) = (Settings.BackgroundLinks, Settings.BenchmarkLinks);

void Run<TStore>(string group, string id, Sampling sampling, Func<TStore> open, Work<TStore>? work)
    where TStore : IBenchedLinks
{
    if (work is null || !$"{group}/{id}".Contains(filter, StringComparison.Ordinal))
    {
        return;
    }
    Console.Error.WriteLine($"Benchmarking {group}/{id}");
    using var store = open();
    var estimate = Harness.Measure(sampling, store, work.Operation, work.Undo);
    Console.WriteLine(estimate.Bencher(group, id));
}

if (backend is "" or "neo4j")
{
    foreach (var group in Benchmarks.Groups)
    {
        var single = Benchmarks.Single(group, background, links);
        Run<IBenchedLinks>(group, "Neo4j_NonTransaction", Sampling.Neo4j, () => Neo4jLinks.Open(Mode.AutoCommit), single);
        Run<IBenchedLinks>(group, "Neo4j_Transaction", Sampling.Neo4j, () => Neo4jLinks.Open(Mode.Transaction), single);
        Run(group, "Neo4j_Batch", Sampling.Neo4j, () => Neo4jLinks.Open(Mode.AutoCommit), Benchmarks.Batch(group, background, links));
    }
}

if (backend is "" or "doublets")
{
    foreach (var group in Benchmarks.Groups)
    {
        foreach (var variant in DoubletsLinks.Variants)
        {
            Run<IBenchedLinks>(group, variant, Sampling.Doublets,
                () => DoubletsLinks.Open(variant, Environment.CurrentDirectory), Benchmarks.Single(group, background, links));
        }
    }
}

// Keeps the visited links alive, so reading them cannot be optimized away.
Console.Error.WriteLine($"Visited link ids sum to {Benchmarks.Visited}");
