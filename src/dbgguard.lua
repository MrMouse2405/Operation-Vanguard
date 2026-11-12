-- Optional editor/reference contract. Runtime supplies this global only with --dbg.
-- Do not require this file. All calls write agent-authored drawings, never truth.
---@class DebugGuard
dbgguard = {}
---Create a named view; existing contents are retained.
function dbgguard.view(name) end
---Upsert node metadata (replaces prior metadata). IDs: string or positive integer.
function dbgguard.node(view, id, info) end
---Upsert an edge; both endpoints must already exist in this agent's view.
function dbgguard.edge(view, id, from, to, info) end
---Replace view-level notes.
function dbgguard.note(view, info) end
---Remove a node plus its incident edges.
function dbgguard.remove_node(view, id) end
---Remove an edge; absent IDs are harmless.
function dbgguard.remove_edge(view, id) end
---Empty contents and notes, retaining the view.
function dbgguard.clear(view) end
