Send me the seed and the replay. “It worked last time” leaves rather a lot of possibilities open.

For a first defence, I want a plan you can explain from the log: **contain → check → cure → reconnect**. Which host did you isolate? What evidence justified it? When will it go back into service? If you cannot answer the last one, you've written quarantine, not recovery.

Scans age. Messages arrive later. Two handlers can want the same slot. Make repeated decisions safe, then test several seeds. One good run may just mean the botnet was kind enough to attack somewhere else.

## First, stop repairing hosts into the same attack

Inside your code, where `host_id` is a discovered local host:

```lua
local function act(action, target)
    local task = {action=action, target=target}
    local ok, reason = vanguard.can_deploy(task)
    if not ok then return false, reason end
    local id = vanguard.deploy(task)
    vanguard.await_any({id})
    if not vanguard.is_ready(id) then
        return false, vanguard.status(id).failure
    end
    return true
end

local ok = act(vanguard.shield.ISOLATE_HOST, host_id)
if ok then ok = act(vanguard.act.RESTORE, host_id) end
if ok then
    print("Clean, still isolated", host_id)
    -- Check the risk before calling RELEASE_HOST.
end
```

Router blocks don’t stop infected neighbours in the same LAN. Use host rules or isolation for that.

## Give the other four slots something to do

```lua
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
local pending = {}
for _, host in ipairs(map.hosts) do
    if #pending == vanguard.MAX_TASKS then
        local done = vanguard.await_any(pending)
        print(vanguard.status(done))
        for i, id in ipairs(pending) do
            if id == done then table.remove(pending, i); break end
        end
    end
    local task = {action=vanguard.sense.HOST_SCAN, target=host.id}
    if vanguard.can_deploy(task) then
        table.insert(pending, vanguard.deploy(task))
    end
end
for _, id in ipairs(pending) do
    local ok, result = pcall(function() return vanguard.await(id) end)
    print(ok, result)
end
```

Handlers share your slots. Preflight checks help, but still handle errors if another piece of code takes the last slot.

## Send evidence someone else can use

```lua
vanguard.rx(function(sender, message)
    print("Evidence from", sender, message)
end)
local result = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN}))
for teammate in pairs(vanguard.others()) do
    vanguard.tx(teammate, {kind="discovery", tick=vanguard.ticks(), data=result.data})
end
```

Send host IDs and observation ticks. “Suspicious over here” is a conversation starter, not an actionable message. Register handlers early; give topic subscriptions a tick to commit. Teammates can share their evidence, but your local work is still yours to do.

## Change one thing. Run it again.

```sh
vanguard --script agent.lua --network network.toml --headless --ticks 1000 --replay run.json
```

Read `snapshot.score`. Find where the points went: lost service, infected host-time, or machines you never brought back. Compare against doing nothing and isolating everything. Those are useful baselines because their mistakes are easy to name.

Change one decision, rerun the same seeds, and keep the version that earns its improvement. Rename the file after that. Calling it `defender_final_really_final.lua` has no measurable effect.
