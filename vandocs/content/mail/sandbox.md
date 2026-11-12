No shell. No sockets. No filesystem rummaging. I know what the next question was going to be.

Your Lua runs in a **sandbox**, with `vanguard` already provided. `src/api.lua` is the reference; trying to load it as your agent does not install the API twice.

## What I did leave in

Use tables, loops, functions, `pcall`, basic math, table helpers, and bounded string helpers. One detail to remember: **`string.find` searches literal text, not patterns**.

Your agent can’t read the filesystem, start processes, open network sockets, check a real-world clock, or use an uncontrolled random generator. There’s no manual coroutine API. Vanguard’s messages and waits handle that work for you.

## Modules are allowed. Put them here.

Put a module beside your entrypoint:

```lua
-- team.lua
return {topic="threats"}
```

Then load it:

```lua
-- agent.lua
local team = require("team")
print(team.topic)
```

Each router has its **own** globals and module cache. Files are loaded together at startup, so every router gets the same source files. Restart reloads your edits and creates fresh VMs (separate running copies of Lua).

## You can have print, too

```lua
print("scan", {host=1001, assessment="SUSPICIOUS"})
```

Open **Output** in the dashboard. Values expand, and each entry shows the agent and file it came from. Huge values and tables that refer back to themselves are safely shortened.

Printing uses fuel. Log the host, the tick, and the decision. A thousand copies of “still running” tell Debug precisely one thing, and it is not why your agent lost.
