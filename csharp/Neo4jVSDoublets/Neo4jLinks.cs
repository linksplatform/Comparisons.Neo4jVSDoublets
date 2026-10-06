using Neo4j.Driver;
using Platform.Data;

namespace Comparisons.Neo4jVSDoublets;

/// <summary>How statements are grouped into transactions.</summary>
public enum Mode
{
    /// <summary>Every statement is a separate, automatically committed transaction.</summary>
    AutoCommit,

    /// <summary>Statements run in one explicit transaction until <see cref="Neo4jLinks.Commit"/>.</summary>
    Transaction,
}

/// <summary>
/// A links store kept in a Neo4j database, reached through the official
/// <see href="https://www.nuget.org/packages/Neo4j.Driver">Neo4j.Driver</see>.
/// It runs the same Cypher statements as the Rust benchmarks (see <c>rust/src/neo4j_impl.rs</c>).
/// </summary>
/// <remarks>
/// A link is a node <c>(:Link {id: Integer, source: Integer, target: Integer})</c>, found by the <c>id</c>
/// property, which is backed by a uniqueness constraint. New ids come from a counter kept by the client,
/// as with a database sequence, so both databases assign the same ids: <c>1, 2, 3, ...</c>, and deleting
/// the link with the highest id makes that id the next one to be created.
///
/// The driver is opened once and its single session is reused by all operations, so no connection is
/// opened per operation. All settings of the driver and of the server are defaults. The driver is
/// asynchronous; every call waits for its result, as the Rust benchmark does.
/// </remarks>
public sealed class Neo4jLinks : IBenchedLinks
{
    /// <summary>Number of links created or deleted per transaction by <see cref="Fork"/> and <see cref="Clear"/>.</summary>
    public const int BatchSize = 10_000;

    private static readonly string[] Parts = ["id", "source", "target"];

    private readonly IDriver _driver;
    private readonly IAsyncSession _session;
    private IAsyncTransaction? _transaction;
    private ulong _nextId = 1;

    private Neo4jLinks(IDriver driver, Mode mode)
    {
        _driver = driver;
        _session = driver.AsyncSession();
        Mode = mode;
    }

    public Mode Mode { get; }

    /// <summary>The same value as in the Doublets stores.</summary>
    public ulong Any { get; } = new LinksConstants<ulong>(enableExternalReferencesSupport: false).Any;

    /// <summary>Opens the driver and creates the schema if it does not exist.</summary>
    public static Neo4jLinks Connect(string uri, string user, string password, Mode mode)
    {
        var store = new Neo4jLinks(GraphDatabase.Driver(uri, AuthTokens.Basic(user, password)), mode);
        foreach (var statement in new[]
        {
            "CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE",
            "CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)",
            "CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)",
            "CALL db.awaitIndexes()",
        })
        {
            store.Fetch(statement);
        }
        return store;
    }

    /// <summary>
    /// Opens the connection described by the <c>NEO4J_URI</c>, <c>NEO4J_USER</c> and <c>NEO4J_PASSWORD</c>
    /// environment variables (defaults: <c>bolt://localhost:7687</c>, <c>neo4j</c>, <c>password</c>),
    /// and removes all links.
    /// </summary>
    public static Neo4jLinks Open(Mode mode)
    {
        var store = Connect(
            Settings.Variable("NEO4J_URI", "bolt://localhost:7687"),
            Settings.Variable("NEO4J_USER", "neo4j"),
            Settings.Variable("NEO4J_PASSWORD", "password"),
            mode);
        store.Clear();
        return store;
    }

    /// <summary>
    /// Creates <paramref name="backgroundLinks"/> point links with the next ids, committing every
    /// <see cref="BatchSize"/> links, so that millions of links do not have to fit into the memory of one transaction.
    /// </summary>
    public void Fork(int backgroundLinks)
    {
        var first = (long)_nextId;
        var last = first + backgroundLinks - 1;
        Fetch(
            "UNWIND range($first, $last) AS id " +
            "CALL (id) { CREATE (:Link {id: id, source: id, target: id}) } " +
            $"IN TRANSACTIONS OF {BatchSize} ROWS",
            new { first, last });
        _nextId = (ulong)(last + 1);
    }

    public void Unfork() => Clear();

    /// <summary>Starts an explicit transaction when the store is in <see cref="Mode.Transaction"/>.</summary>
    public void Begin()
    {
        if (Mode == Mode.Transaction)
        {
            _transaction = Wait(_session.BeginTransactionAsync());
        }
    }

    /// <summary>Commits the explicit transaction started by <see cref="Begin"/>, if any.</summary>
    public void Commit()
    {
        if (_transaction is { } transaction)
        {
            _transaction = null;
            Wait(transaction.CommitAsync());
        }
    }

    /// <summary>Rolls back the open transaction, if any, and deletes all links, committing every <see cref="BatchSize"/> links.</summary>
    public void Clear()
    {
        if (_transaction is { } transaction)
        {
            _transaction = null;
            Wait(transaction.RollbackAsync());
        }
        Fetch($"MATCH (l:Link) CALL (l) {{ DELETE l }} IN TRANSACTIONS OF {BatchSize} ROWS");
        _nextId = 1;
    }

    public ulong CreatePoint()
    {
        var id = _nextId;
        Fetch("CREATE (:Link {id: $id, source: $id, target: $id})", new { id = (long)id });
        _nextId = id + 1;
        return id;
    }

    public void Update(ulong id, ulong source, ulong target)
    {
        var records = Fetch(
            "MATCH (l:Link {id: $id}) " +
            "WITH l, l.source AS source, l.target AS target " +
            "SET l.source = $new_source, l.target = $new_target " +
            "RETURN source, target",
            new { id = (long)id, new_source = (long)source, new_target = (long)target });
        Expect(records.Count == 1, $"link {id} does not exist");
    }

    public void Delete(ulong id)
    {
        var records = Fetch(
            "MATCH (l:Link {id: $id}) " +
            "WITH l, l.source AS source, l.target AS target " +
            "DELETE l " +
            "RETURN source, target",
            new { id = (long)id });
        Expect(records.Count == 1, $"link {id} does not exist");
        if (id + 1 == _nextId)
        {
            _nextId = id;
        }
    }

    public void Each(ulong id, ulong source, ulong target, Action<Link> visit)
    {
        ulong[] query = [id, source, target];
        var parameters = new Dictionary<string, object>();
        for (var part = 0; part < Parts.Length; part++)
        {
            if (query[part] != Any)
            {
                parameters[Parts[part]] = (long)query[part];
            }
        }
        var filter = Filter(query, part => $"${Parts[part]}");
        Fetch($"MATCH (l:Link){filter} RETURN l.id AS id, l.source AS source, l.target AS target", parameters,
            record => visit(LinkOf(record)));
    }

    public ulong Count() => (ulong)Fetch("MATCH (l:Link) RETURN count(l) AS count")[0]["count"].As<long>();

    // ------------------------------------------------------------------
    // Batches: one statement for many links (the `Neo4j_Batch` benchmarks)
    // ------------------------------------------------------------------

    /// <summary>Creates the point links with the next <paramref name="count"/> ids with one statement.</summary>
    public void CreatePointsBatch(int count)
    {
        var first = (long)_nextId;
        var last = first + count - 1;
        Fetch("UNWIND range($first, $last) AS id CREATE (:Link {id: id, source: id, target: id})", new { first, last });
        _nextId = (ulong)(last + 1);
    }

    /// <summary>Sets <c>source</c> and <c>target</c> of every link in <paramref name="links"/> (found by its id) with one statement.</summary>
    public void UpdateBatch(IReadOnlyList<Link> links)
    {
        var records = Fetch(
            "UNWIND $links AS link " +
            "MATCH (l:Link {id: link[0]}) " +
            "SET l.source = link[1], l.target = link[2] " +
            "RETURN count(l) AS updated",
            new { links = links.Select(link => new[] { (long)link.Id, (long)link.Source, (long)link.Target }).ToList() });
        ExpectCount(records, "updated", links.Count);
    }

    /// <summary>Deletes the links with the given ids with one statement.</summary>
    public void DeleteBatch(IReadOnlyList<ulong> ids)
    {
        var records = Fetch(
            "UNWIND $ids AS id " +
            "MATCH (l:Link {id: id}) " +
            "DELETE l " +
            "RETURN count(l) AS deleted",
            new { ids = ids.Select(id => (long)id).ToList() });
        ExpectCount(records, "deleted", ids.Count);
        // The same rule as in `Delete`, applied from the highest id.
        foreach (var id in ids.OrderDescending())
        {
            if (id + 1 == _nextId)
            {
                _nextId = id;
            }
        }
    }

    /// <summary>
    /// Runs all <paramref name="queries"/> (<c>[id, source, target]</c>) with one statement and passes every found link
    /// to <paramref name="visit"/>. All queries must constrain the same parts, because they share one <c>WHERE</c> clause.
    /// </summary>
    public void EachBatch(IReadOnlyList<ulong[]> queries, Action<Link> visit)
    {
        if (queries.Count == 0)
        {
            return;
        }
        var first = queries[0];
        if (queries.Any(query => Enumerable.Range(0, 3).Any(part => (query[part] == Any) != (first[part] == Any))))
        {
            throw new ArgumentException("all queries of a batch must constrain the same parts", nameof(queries));
        }
        // Only the constrained parts are sent: `q[0]` is the first of them.
        var parts = Enumerable.Range(0, 3).Where(part => first[part] != Any).ToList();
        var filter = Filter(first, part => $"q[{parts.IndexOf(part)}]");
        Fetch(
            $"UNWIND $queries AS q MATCH (l:Link){filter} RETURN l.id AS id, l.source AS source, l.target AS target",
            new Dictionary<string, object>
            {
                ["queries"] = queries.Select(query => parts.Select(part => (long)query[part]).ToList()).ToList(),
            },
            record => visit(LinkOf(record)));
    }

    public void Dispose()
    {
        Clear();
        Wait(_session.CloseAsync());
        _driver.Dispose();
    }

    /// <summary>
    /// Builds the <c> WHERE ...</c> clause with one condition per part of <paramref name="query"/> that is not
    /// <see cref="Any"/>; <paramref name="value"/> returns the Cypher expression the property is compared with.
    /// </summary>
    private string Filter(ulong[] query, Func<int, string> value)
    {
        var conditions = Enumerable.Range(0, 3)
            .Where(part => query[part] != Any)
            .Select(part => $"l.{Parts[part]} = {value(part)}")
            .ToList();
        return conditions.Count == 0 ? "" : $" WHERE {string.Join(" AND ", conditions)}";
    }

    private List<IRecord> Fetch(string query, object? parameters = null)
    {
        var records = new List<IRecord>();
        Fetch(query, parameters, records.Add);
        return records;
    }

    /// <summary>Runs <paramref name="query"/> in the open transaction (or as an auto-commit transaction) and passes every returned record to <paramref name="onRecord"/>.</summary>
    private void Fetch(string query, object? parameters, Action<IRecord> onRecord) => Wait(FetchAsync(query, parameters, onRecord));

    private async Task FetchAsync(string query, object? parameters, Action<IRecord> onRecord)
    {
        var statement = parameters switch
        {
            null => new Query(query),
            IDictionary<string, object> dictionary => new Query(query, dictionary),
            _ => new Query(query, parameters),
        };
        var cursor = _transaction is { } transaction
            ? await transaction.RunAsync(statement)
            : await _session.RunAsync(statement);
        while (await cursor.FetchAsync())
        {
            onRecord(cursor.Current);
        }
    }

    private static Link LinkOf(IRecord record) =>
        new((ulong)record["id"].As<long>(), (ulong)record["source"].As<long>(), (ulong)record["target"].As<long>());

    private static void ExpectCount(List<IRecord> records, string column, int expected)
    {
        var count = records[0][column].As<long>();
        Expect(count == expected, $"{count} links {column}, expected {expected}");
    }

    private static void Expect(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    private static T Wait<T>(Task<T> task) => task.GetAwaiter().GetResult();

    private static void Wait(Task task) => task.GetAwaiter().GetResult();
}
