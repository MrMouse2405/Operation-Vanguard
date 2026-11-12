Split your policy into helpers you can read without scrolling through an entire seven-hour crisis.

## Define, call, return

```lua
local function is_risky(assessment)
    return assessment == "SUSPICIOUS" or assessment == "COMPROMISED"
end
print(is_risky("CLEAN")) -- false

local allowed, reason = vanguard.can_deploy({action=vanguard.sense.SCAN})
if not allowed then print("Not now", reason) end
```

Functions can return several values. `can_deploy` returns true, or false plus a reason. Do not turn that reason into an imaginary task ID.

## Catch a raised error

```lua
local ok, value = pcall(function()
    return vanguard.deploy({action=vanguard.sense.SCAN})
end)
if ok then
    local result = vanguard.await(value)
    print(result.data)
else
    print("Admission failed", value)
end
```

This catches admission errors. A later task failure is a separate event: `await` raises it. Use `await_any`, then `is_ready` and `status(id).failure` when you want to inspect failure as data. [Task calls](/docs/api/tasks/) explain the difference.

## A callback runs later

```lua
local reports = {}
vanguard.rx(function(sender, message, metadata)
    -- Registered now; called by the engine when a message arrives later.
    reports[sender] = {message=message, received=metadata.delivered_at}
end)
```

This function remembers the local `reports` table: a closure. It runs in its own coroutine but shares this router’s memory, fuel and task slots. Keep handlers short; enqueue advice and let your main loop perform mutations if you want fewer lock conflicts.

Modules use `require("team")` for `team.lua` beside your entrypoint. Each router has its own module cache. [Sandbox’s note](/docs/sandbox/) shows a complete two-file example. The API already exists as `vanguard`; `src/api.lua` is a reference, not a module to load.
