I have two registers: **agents** and **nodes**. Both use numbers. Sending an agent message to a router number is still addressing the wrong register. Please keep the headings on your variables.

## vanguard.self

`vanguard.self() → integer`

Your agent’s entry in the agent register. Use it for messaging identity and fuel queries. Do not put it in a host task's `target` field.

## vanguard.router

`vanguard.router() → integer`

The node ID of the router running your code. Router-only actions already know this target, so don’t put it in their descriptor: an explicit target is rejected.

## vanguard.others

`vanguard.others() → table<agent_id, router_id>`

A map from active teammate agent IDs to their router node IDs. Your own entry is omitted. File it as **agent → router**, not an array of records: use `pairs`, not `ipairs`. Calculate neither number from the other; that is why I provide the map.

```lua
print("Me", vanguard.self(), "my router", vanguard.router())
for agent_id, router_id in pairs(vanguard.others()) do
    print("Teammate", agent_id, "on router", router_id)
end
```

`agent_id` and `router_id` are longer names than `id`. In a message handler, I consider the extra typing a worthwhile expense. Scalar keys have a stable iteration order here; your strategy should not need that order to mean anything.
