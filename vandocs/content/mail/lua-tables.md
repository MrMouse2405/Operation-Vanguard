Lua uses **tables** for lists and maps. The API uses both. That is convenient until someone iterates a map as if it were a list and concludes there are no teammates.

## Lists start at one

```lua
local hosts = {1001, 1002, 1003}
print(hosts[1]) -- 1001, not the second element
print(#hosts)  -- 3, for a dense list
for index, host in ipairs(hosts) do
    print(index, host)
end
table.insert(hosts, 1004)
table.remove(hosts, 1) -- shifts later items down
```

`ipairs` stops at the first missing integer position. Do not make holes in lists used for `await_any` or message payloads.

## Maps have named keys

```lua
local report = {host=1001, assessment="SUSPICIOUS", observed_at=7}
print(report.host) -- same as report["host"]
report.assessment = "COMPROMISED"
for field, value in pairs(report) do print(field, value) end

local teammate_routers = vanguard.others()
for agent_id, router_id in pairs(teammate_routers) do
    print("agent", agent_id, "router", router_id)
end
```

Use `pairs` for `others()`: it is a map of agent ID → router ID, not a dense list. Avoid repeatedly iterating a giant map just to retrieve one known key. `map[id]` is clearer and cheaper.

## References are not copies

```lua
local original = {host=1001}
local alias = original
alias.host = 1002
print(original.host) -- 1002: same local table
```

Messages are different: the runtime copies the payload when you send it. API result tables are snapshots too. Editing your local scan result does not alter the machine. Assigning `scan.assessment = "CLEAN"` is paperwork, not medicine.

Payloads must be a tree: dense arrays or string-keyed objects, not mixed keys, cycles, functions or metatables. The [messaging email](/docs/messaging/) covers limits. Next: [functions and failures](/docs/lua/functions/).
