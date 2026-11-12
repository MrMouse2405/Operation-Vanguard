Lua is small. We can learn enough syntax before the coffee gets cold. All examples here run inside Vanguard; no filesystem, sockets or real-world clock required.

## Values and decisions

```lua
local host_id = 1001       -- local variable; comment starts with --
local assessment = "CLEAN" -- strings use quotes
local protected = false   -- booleans: true and false
local missing = nil       -- absent value

if assessment ~= "CLEAN" then -- ~= means NOT equal
    print("Needs attention", host_id)
elseif protected then
    print("Already protected")
else
    print("No action yet")
end
```

`=` assigns. `==` compares. `~=` compares for inequality. `and`, `or`, `not` combine conditions. **Only false and nil are falsey.** Zero and an empty string both count as true. This surprises people migrating from Python; consider this the official surprise notification.

## Loops

```lua
for tick = 1, 3 do
    print("loop", tick) -- 1, 2, 3: the upper bound is included
end

local checks = 0
while checks < 3 do
    checks = checks + 1 -- Lua has no += operator
end
print("host-" .. 1001) -- .. joins strings
```

An endless loop consumes fuel; it does not freeze the simulator. For a periodic agent loop, call `vanguard.wait(1)` rather than repeatedly checking the same tick. **Wait pauses this coroutine; tasks and other handlers can continue.**

## Small practice changes

In the supplied sensing agent, change a printed label, print the current tick, and print only servers using `host.type == "SERVER"`. You should understand each change before testing it. Next: [tables](/docs/lua/tables/) — most API results are tables.
