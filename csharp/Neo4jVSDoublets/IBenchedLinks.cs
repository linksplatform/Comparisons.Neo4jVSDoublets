namespace Comparisons.Neo4jVSDoublets;

/// <summary>A link: its id, the link it starts from (source) and the link it ends at (target).</summary>
public readonly record struct Link(ulong Id, ulong Source, ulong Target)
{
    public static Link Point(ulong id) => new(id, id, id);
}

/// <summary>
/// A links store that can be benchmarked. Neo4j and every Doublets store implement it,
/// so every benchmark calls exactly the same methods on both databases.
/// </summary>
/// <remarks>
/// A benchmark of one operation is:
/// <list type="number">
/// <item><see cref="Fork"/> - not measured: create the background links</item>
/// <item>for every iteration: <see cref="Begin"/>, the operation, <see cref="Commit"/> - measured;
/// then undo the changes of the operation - not measured</item>
/// <item><see cref="Unfork"/> - not measured: remove all links</item>
/// </list>
/// </remarks>
public interface IBenchedLinks : IDisposable
{
    /// <summary>The value of a query part that matches every value.</summary>
    ulong Any { get; }

    /// <summary>Creates the point links <c>1..=backgroundLinks</c> (each with <c>id = source = target</c>) in the empty store.</summary>
    void Fork(int backgroundLinks);

    /// <summary>Starts a transaction. Only Neo4j in transaction mode has one.</summary>
    void Begin()
    {
    }

    /// <summary>Commits the transaction started by <see cref="Begin"/>. For all other stores every operation is already complete when it returns.</summary>
    void Commit()
    {
    }

    /// <summary>Removes all links.</summary>
    void Unfork();

    /// <summary>Creates a link with <c>id = source = target</c> and returns its id.</summary>
    ulong CreatePoint();

    /// <summary>Sets the source and the target of the link <paramref name="id"/>.</summary>
    void Update(ulong id, ulong source, ulong target);

    /// <summary>Deletes the link <paramref name="id"/>.</summary>
    void Delete(ulong id);

    /// <summary>Passes every link that matches the query to <paramref name="visit"/>; <see cref="Any"/> matches every value.</summary>
    void Each(ulong id, ulong source, ulong target, Action<Link> visit);

    /// <summary>Returns the number of links.</summary>
    ulong Count();
}
