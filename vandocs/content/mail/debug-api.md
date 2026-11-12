`dbgguard` is a **write-only drawing API**. Your script creates representations of its discoveries, connections, estimates and notes. The dashboard displays them side by side. It does not expose hidden simulation state to Lua.

```sh
vanguard --script vanguard/agents/examples/debug-map.lua --network vanguard/networks/tutorial-2.toml --dbg
```

Without `--dbg`, the global is **nil**. Guard optional instrumentation with `if dbgguard then ... end`. The flag persists through dashboard restarts; each fresh run starts with empty drawings. It works in headless mode too: final views are in `snapshot.debug.agents`.

## Seven calls, all returning true

| Call | Effect |
| --- | --- |
| `dbgguard.view(name)` | Create a view, or keep an existing one |
| `dbgguard.node(view, id, info)` | Insert or replace a node and its attached information |
| `dbgguard.edge(view, id, from, to, info)` | Insert or replace an edge; both nodes must exist in that view |
| `dbgguard.note(view, info)` | Replace the view’s notes |
| `dbgguard.remove_node(view, id)` | Remove a node and its incident edges; missing ID is harmless |
| `dbgguard.remove_edge(view, id)` | Remove an edge; missing ID is harmless |
| `dbgguard.clear(view)` | Empty nodes, edges and notes; retain the view |

Create the view first. View names and IDs are nonempty strings of at most 128 bytes, or positive integers. Integer `1001` and string `"1001"` name the same drawing node. Edge IDs distinguish parallel edges. `from` → `to` is retained in edge records; the compact graph uses a connecting line, with direction in the edge details.

`info` can be any supported copied JSON-like payload: text, a finite number, boolean, dense list, or string-keyed object. Use `observed_at`, `confidence`, `source`, `assessment` or any other fields useful to your investigation. A node upsert **replaces**, rather than merges, its info.

## A complete tiny drawing

```lua
if dbgguard then
    dbgguard.view("hypothesis")
    dbgguard.node("hypothesis", "router", {label="My router"})
    dbgguard.node("hypothesis", "unknown", {assessment="UNKNOWN"})
    dbgguard.edge("hypothesis", "guess", "router", "unknown", {confidence="low"})
    dbgguard.note("hypothesis", {explanation="A drawing, not a discovered fact"})
    dbgguard.remove_edge("hypothesis", "guess")
    dbgguard.remove_node("hypothesis", "unknown")
    dbgguard.clear("hypothesis") -- empty view remains selectable
end
```

## Budgets and failure behaviour

Normal instruction fuel applies. Each drawing call also pays the native-call cost plus serialized argument bytes, and must fit `max_payload_bytes`. It creates no task, takes no mutation lock, and cannot change world state or message teammates. Instrumentation can delay agent decisions through fuel use; compare scored agents under the same instrumentation settings.

Per agent: **8 views, 4,096 nodes, 8,192 edges, 1 MiB serialized state total**. Invalid IDs, counts, missing views/endpoints or unsupported payloads raise `INVALID_ARGUMENT`; payload overflow raises `PAYLOAD_TOO_LARGE`; store limits raise `LIMIT_EXCEEDED`. Rejected drawing updates leave the previous drawing intact. Native fuel exhaustion yields/retries like other calls.

The panes paginate at 80 nodes. They show the latest drawing, not an automatic historical slider. Make two explicitly named views to compare hypotheses or snapshots. Data is namespaced by agent; two routers may both call a view “discovery” without overwriting each other.

Next: [reconstruct a real observed LAN](/docs/debug/reconstruction/).
