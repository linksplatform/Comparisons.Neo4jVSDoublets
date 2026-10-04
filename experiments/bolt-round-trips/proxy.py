"""Counts Bolt round trips: a TCP proxy (localhost:7688 -> localhost:7687) that prints (on SIGUSR1)
the number of client->server "flights" (consecutive client writes not interrupted by a
server reply). Each flight followed by a server reply is one network round trip."""
import selectors, signal, socket, sys, threading

LISTEN, TARGET = ("127.0.0.1", 7688), ("127.0.0.1", 7687)
VERBOSE = len(sys.argv) > 1 and sys.argv[1] == "--verbose"
NAMES = {0x01: "HELLO", 0x0F: "RESET", 0x10: "RUN", 0x11: "BEGIN", 0x12: "COMMIT",
         0x13: "ROLLBACK", 0x2F: "DISCARD", 0x3F: "PULL", 0x54: "TELEMETRY", 0x6A: "LOGON"}
flights = 0


def describe(data):
    """Names the Bolt messages in a chunk of client bytes (best effort, for tracing only)."""
    names, i = [], 0
    while i + 2 <= len(data):
        size = int.from_bytes(data[i:i + 2], "big")
        if size and i + 3 < len(data) and data[i + 2] & 0xF0 == 0xB0:
            names.append(NAMES.get(data[i + 3], hex(data[i + 3])))
        i += 2 + size
        while i + 2 <= len(data) and data[i:i + 2] == b"\0\0":
            i += 2
    return names
lock = threading.Lock()

def handle(client):
    global flights
    server = socket.create_connection(TARGET)
    for s in (client, server):
        s.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    sel = selectors.DefaultSelector()
    sel.register(client, selectors.EVENT_READ, server)
    sel.register(server, selectors.EVENT_READ, client)
    last = None
    while True:
        for key, _ in sel.select():
            data = key.fileobj.recv(65536)
            if not data:
                return
            if key.fileobj is client and last is not client:
                with lock:
                    flights += 1
            if key.fileobj is client and VERBOSE:
                print("client:", describe(data), flush=True)
            last = key.fileobj
            key.data.sendall(data)

def report(*_):
    print(f"flights={flights}", flush=True)

signal.signal(signal.SIGUSR1, report)

listener = socket.socket()
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(LISTEN)
listener.listen()
while True:
    conn, _ = listener.accept()
    threading.Thread(target=handle, args=(conn,), daemon=True).start()
