Some RMC kid cooked up a simulator and called it **Operation Vanguard**. The vendor gets a logo; we get to explain why turning every server off is not defence.

Your task: write the Lua program running inside the router modules, stop a simulated botnet, and keep required workstation-to-server connections useful. [Scoreboard](/docs/scoring/) has the arithmetic: **600 service + 250 security + 150 final recovery**.

## Fuel: less magic than the rumour suggests

The simulator counts **Lua bytecode instructions**, plus charges for API calls, payload handling and admitted tasks. That work is called **fuel**. Every router gets a fresh, equal fuel allowance each tick. Main code and message handlers share it.

The rumour says “efficient algorithms act faster than the botnet.” The precise version is: **waste enough computation and your next decision slips into a later tick**, while the botnet continues at turn boundaries. Efficient code can admit useful tasks earlier. Fuel is not a second clock that speeds up the enemy, and unused fuel does not shorten task durations.

```lua
-- Good: ask for a result, then sleep until it is available.
local id = vanguard.deploy({action=vanguard.sense.SCAN})
local data = vanguard.await(id).data
print(data)
```

```lua
-- Wasteful: this loop does work but cannot finish the task sooner.
local id = vanguard.deploy({action=vanguard.sense.SCAN})
while not vanguard.is_finished(id) do
    -- Repeated checks consume fuel. Use await instead.
end
```

Both still need the scan’s **2 ticks**. The second just works harder at waiting. [Clock](/docs/ticks/) is available for further disappointment.

## The world in six sentences

1. Routers are immune. Workstations and servers are not.
2. Every router owns one LAN and can change only its own hosts.
3. A compromised online host attacks up to two targets; infection needs **14 uninterrupted ticks** by default.
4. Isolation interrupts traffic. Shutdown stops traffic too. Neither cures infection.
5. **RESTORE takes 20 ticks**, cures the host and returns it online, retaining rules and isolation.
6. Required service must survive: **30 consecutive ticks with no working required connections means defeat** by default.

## What makes a run reproducible?

The same agent sources, network, seed, mission and budgets produce deterministic barriers. Browser playback speed changes how long you watch, not the game rules. Restart reloads both the script and network into fresh VMs. Headless runs use the same barriers.

```sh
vanguard --script vanguard/agents/basic-defender.lua --network vanguard/networks/tutorial-1.toml --headless --ticks 300 --replay run.json
```

The final score is at `snapshot.score`. The replay also records semantic hashes. Compare several seeds and topologies; a single victory proves that one run worked. It does not prove your code has achieved enlightenment.

Next: [Tutorial 1](/docs/tutorial/one/). You can have a dashboard open in five minutes.
