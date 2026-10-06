using Platform.Data;
using Platform.Data.Doublets;
using Platform.Data.Doublets.Memory.Split.Generic;
using Platform.Data.Doublets.Memory.United.Generic;
using Platform.Memory;

namespace Comparisons.Neo4jVSDoublets;

/// <summary>
/// A Doublets store of <see href="https://www.nuget.org/packages/Platform.Data.Doublets">Platform.Data.Doublets</see>.
/// The store runs inside the benchmark process: every operation reads and writes the store's memory directly,
/// without a server, a network connection or a query language.
/// </summary>
/// <remarks>
/// <list type="table">
/// <item><term><c>Doublets_United_Volatile</c></term><description><c>UnitedMemoryLinks</c> in RAM</description></item>
/// <item><term><c>Doublets_United_NonVolatile</c></term><description><c>UnitedMemoryLinks</c> in a memory-mapped file</description></item>
/// <item><term><c>Doublets_Split_Volatile</c></term><description><c>SplitMemoryLinks</c> in RAM</description></item>
/// <item><term><c>Doublets_Split_NonVolatile</c></term><description><c>SplitMemoryLinks</c> in two memory-mapped files</description></item>
/// </list>
/// Writes to a memory-mapped file go to the OS page cache; the benchmarks do not flush them to the disk.
/// </remarks>
public sealed class DoubletsLinks(ILinks<ulong> links) : IBenchedLinks
{
    public static readonly string[] Variants =
    [
        "Doublets_United_Volatile",
        "Doublets_United_NonVolatile",
        "Doublets_Split_Volatile",
        "Doublets_Split_NonVolatile",
    ];

    private readonly ulong _continue = links.Constants.Continue;

    public ulong Any { get; } = links.Constants.Any;

    /// <summary>Opens the store of <paramref name="variant"/>; the non-volatile stores are kept in <paramref name="directory"/>.</summary>
    public static DoubletsLinks Open(string variant, string directory)
    {
        static FileMappedResizableDirectMemory Mapped(string directory, string name)
        {
            var path = Path.Combine(directory, name);
            File.Delete(path);
            return new FileMappedResizableDirectMemory(path);
        }

        ILinks<ulong> links = variant switch
        {
            "Doublets_United_Volatile" => new UnitedMemoryLinks<ulong>(new HeapResizableDirectMemory()),
            "Doublets_United_NonVolatile" => new UnitedMemoryLinks<ulong>(Mapped(directory, "united.links")),
            "Doublets_Split_Volatile" => new SplitMemoryLinks<ulong>(new HeapResizableDirectMemory(), new HeapResizableDirectMemory()),
            "Doublets_Split_NonVolatile" => new SplitMemoryLinks<ulong>(Mapped(directory, "split_data.links"), Mapped(directory, "split_index.links")),
            _ => throw new ArgumentException($"unknown Doublets store {variant}", nameof(variant)),
        };
        return new DoubletsLinks(links);
    }

    public void Fork(int backgroundLinks)
    {
        for (var i = 0; i < backgroundLinks; i++)
        {
            links.CreatePoint();
        }
    }

    /// <summary>Deletes the links from the highest id down, so the next created link gets id 1 again.</summary>
    public void Unfork()
    {
        var ids = new List<ulong>();
        Each(Any, Any, Any, link => ids.Add(link.Id));
        foreach (var id in ids.OrderDescending())
        {
            Delete(id);
        }
    }

    public ulong CreatePoint() => links.CreatePoint();

    public void Update(ulong id, ulong source, ulong target) => links.Update(id, source, target);

    /// <summary>
    /// Resets the link to <c>(0, 0)</c>, which removes it from the index trees, and then frees it.
    /// </summary>
    public void Delete(ulong id) => links.Delete(id, handler: null);

    public void Each(ulong id, ulong source, ulong target, Action<Link> visit) => links.Each(link =>
    {
        visit(new Link(link![0], link[1], link[2]));
        return _continue;
    }, id, source, target);

    public ulong Count() => links.Count();

    public void Dispose() => (links as IDisposable)?.Dispose();
}
