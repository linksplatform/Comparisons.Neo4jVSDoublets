using Xunit;

namespace Comparisons.Neo4jVSDoublets.Tests;

/// <summary>
/// Checks that every benchmarked store gives the same results for the operations used by the benchmarks,
/// and that undoing an operation (as the benchmarks do after every iteration) restores the background links.
/// The same checks as in <c>rust/tests/same_behavior.rs</c>.
/// </summary>
/// <remarks>
/// The Neo4j tests need a running server and are skipped unless <c>NEO4J_URI</c> is set
/// (for example <c>NEO4J_URI=bolt://localhost:7687 dotnet test</c>).
/// </remarks>
public class SameBehaviorTests
{
    private const int Background = 20;
    private const int Links = 5;

    private static List<Link> Query(IBenchedLinks store, ulong id, ulong source, ulong target)
    {
        var links = new List<Link>();
        store.Each(id, source, target, links.Add);
        return links.OrderBy(link => link.Id).ToList();
    }

    private static List<Link> All(IBenchedLinks store) => Query(store, store.Any, store.Any, store.Any);

    private static List<Link> Points(int first, int last) =>
        Enumerable.Range(first, last - first + 1).Select(id => Link.Point((ulong)id)).ToList();

    private static void InTransaction(IBenchedLinks store, Action action)
    {
        store.Begin();
        action();
        store.Commit();
    }

    /// <summary>Runs the operations of the benchmarks inside <c>Begin</c>/<c>Commit</c> and checks their results.</summary>
    private static void Check(IBenchedLinks store)
    {
        var any = store.Any;
        store.Fork(Background);
        var background = Points(1, Background);
        Assert.Equal(background, All(store));

        // Create, undone by Delete.
        var created = new List<ulong>();
        InTransaction(store, () =>
        {
            for (var i = 0; i < Links; i++)
            {
                created.Add(store.CreatePoint());
            }
        });
        Assert.Equal(Points(Background + 1, Background + Links).Select(link => link.Id), created);
        InTransaction(store, () =>
        {
            for (var id = (ulong)(Background + Links); id > Background; id--)
            {
                store.Delete(id);
            }
        });
        Assert.Equal(background, All(store));

        // Delete, undone by Create.
        InTransaction(store, () =>
        {
            for (var id = (ulong)Background; id > Background - Links; id--)
            {
                store.Delete(id);
            }
        });
        Assert.Equal(Points(1, Background - Links), All(store));
        InTransaction(store, () =>
        {
            for (var i = 0; i < Links; i++)
            {
                store.CreatePoint();
            }
        });
        Assert.Equal(background, All(store));

        // Update, restored by the second update.
        InTransaction(store, () =>
        {
            store.Update(Background, 0, 0);
            Assert.Equal([new Link(Background, 0, 0)], Query(store, Background, any, any));
            store.Update(Background, Background, Background);
        });
        Assert.Equal(background, All(store));

        // The queries of the benchmarks.
        for (ulong id = 1; id <= Background; id++)
        {
            List<Link> point = [Link.Point(id)];
            Assert.Equal(point, Query(store, id, any, any));
            Assert.Equal(point, Query(store, any, id, id));
            Assert.Equal(point, Query(store, any, id, any));
            Assert.Equal(point, Query(store, any, any, id));
        }
        Assert.Equal((ulong)Background, store.Count());

        // The benchmarks run every iteration on a store that `Unfork` emptied and `Fork` filled again.
        store.Unfork();
        Assert.Equal(0UL, store.Count());
        store.Fork(Background);
        Assert.Equal(background, All(store));
        store.Unfork();
    }

    [Theory]
    [InlineData("Doublets_United_Volatile")]
    [InlineData("Doublets_United_NonVolatile")]
    [InlineData("Doublets_Split_Volatile")]
    [InlineData("Doublets_Split_NonVolatile")]
    public void Doublets(string variant)
    {
        var directory = Directory.CreateTempSubdirectory("same_behavior");
        try
        {
            using var store = DoubletsLinks.Open(variant, directory.FullName);
            Check(store);
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact]
    public void DoubletsVariantsAreAllTested() =>
        Assert.Equal(DoubletsLinks.Variants, typeof(SameBehaviorTests).GetMethod(nameof(Doublets))!
            .GetCustomAttributes(typeof(InlineDataAttribute), false)
            .Select(attribute => (string)((InlineDataAttribute)attribute).Data[0]!));

    private static void SkipWithoutNeo4j() =>
        Assert.SkipWhen(string.IsNullOrEmpty(Environment.GetEnvironmentVariable("NEO4J_URI")), "NEO4J_URI is not set");

    /// <summary>All modes share one database, so they run one after another in one test.</summary>
    [Fact]
    public void Neo4j()
    {
        SkipWithoutNeo4j();
        foreach (var mode in new[] { Mode.AutoCommit, Mode.Transaction })
        {
            using var store = Neo4jLinks.Open(mode);
            Check(store);
        }
        using var batch = Neo4jLinks.Open(Mode.AutoCommit);
        CheckBatch(batch);
    }

    /// <summary>
    /// Checks that the batch methods of Neo4j give the same results as the single operations in <see cref="Check"/>,
    /// and that undoing them restores the background links.
    /// </summary>
    private static void CheckBatch(Neo4jLinks store)
    {
        var any = store.Any;
        store.Fork(Background);
        var background = Points(1, Background);

        // Create, undone by Delete.
        store.CreatePointsBatch(Links);
        Assert.Equal(Points(1, Background + Links), All(store));
        store.DeleteBatch(Points(Background + 1, Background + Links).Select(link => link.Id).ToList());
        Assert.Equal(background, All(store));

        // Delete, undone by Create.
        store.DeleteBatch(Points(Background - Links + 1, Background).Select(link => link.Id).ToList());
        Assert.Equal(Points(1, Background - Links), All(store));
        store.CreatePointsBatch(Links);
        Assert.Equal(background, All(store));

        // Update, restored by the second update.
        var zeros = Points(1, Links).Select(link => link with { Source = 0, Target = 0 }).ToList();
        store.UpdateBatch(zeros);
        Assert.Equal(zeros, Query(store, any, 0, 0));
        store.UpdateBatch(Points(1, Links));
        Assert.Equal(background, All(store));

        // A missing link is an error, not silently skipped.
        Assert.Throws<InvalidOperationException>(() => store.DeleteBatch([Background + 1]));
        Assert.Throws<InvalidOperationException>(() => store.UpdateBatch([Link.Point(Background + 1)]));

        // The queries of the benchmarks.
        foreach (var group in new[] { "Each_Identity", "Each_Concrete", "Each_Outgoing", "Each_Incoming" })
        {
            var query = Benchmarks.Query(group)!;
            var queries = background.Select(link => query(link.Id, any) is var (id, source, target) ? new[] { id, source, target } : []).ToList();
            var found = new List<Link>();
            store.EachBatch(queries, found.Add);
            Assert.Equal(background, found.OrderBy(link => link.Id));
        }
        Assert.Throws<ArgumentException>(() => store.EachBatch([[1, any, any], [any, 1, any]], _ => { }));
        store.Unfork();
    }

    /// <summary>Runs every benchmark's work, as the benchmark does, and checks that the store ends as it started.</summary>
    [Fact]
    public void BenchmarkWorkIsUndone()
    {
        foreach (var group in Benchmarks.Groups)
        {
            foreach (var variant in new[] { "Doublets_United_Volatile", "Doublets_Split_Volatile" })
            {
                using var store = DoubletsLinks.Open(variant, Path.GetTempPath());
                store.Fork(Background);
                var work = Benchmarks.Single(group, Background, Links);
                for (var iteration = 0; iteration < 3; iteration++)
                {
                    work.Operation(store);
                    work.Undo(store);
                    Assert.Equal(Points(1, Background), All(store));
                }
            }
        }
    }

    [Fact]
    public void Neo4jBenchmarkWorkIsUndone()
    {
        SkipWithoutNeo4j();
        foreach (var group in Benchmarks.Groups)
        {
            using var store = Neo4jLinks.Open(Mode.Transaction);
            store.Fork(Background);
            var single = Benchmarks.Single(group, Background, Links);
            var batch = Benchmarks.Batch(group, Background, Links);
            for (var iteration = 0; iteration < 2; iteration++)
            {
                InTransaction(store, () => single.Operation(store));
                InTransaction(store, () => single.Undo(store));
                Assert.Equal(Points(1, Background), All(store));
                if (batch is not null)
                {
                    batch.Operation(store);
                    batch.Undo(store);
                    Assert.Equal(Points(1, Background), All(store));
                }
            }
        }
    }
}
