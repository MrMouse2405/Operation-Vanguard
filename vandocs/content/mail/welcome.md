I’ve left a working agent in `vanguard/agents/defender.lua`. Run that before writing your own. It gives you something to compare against when your first experiment manages to disconnect every server.

You supply **one Lua program**. Every router runs its own copy, with private memory. If router A learns something, router B does not magically know it. Relay handles that part.

## Get a run on the board

From the supplied exercise folder, run:

```sh
vanguard --script vanguard/agents/defender.lua --network network.toml
```

For your own agent and network files:

```sh
vanguard --script agent.lua --network network.toml
```

Open **http://127.0.0.1:8000** and press **Start**. Pause freezes the game. **+1** takes one turn; **Forward** takes several. **Restart** reloads your Lua and network files so you can try your edits.

The **s/tick** box sets the minimum length of a turn in seconds, including the time it takes to calculate it. Use **0** for full speed. If a turn takes longer than your setting, there’s no extra wait. This changes playback, not the rules. Manual stepping still runs the exact number of turns you ask for.

## Now replace the agent

Start with this: find the hosts on your LAN and print their assessments. No clever recovery policy yet. First make sure you know which machines you’re looking at.

```lua
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
print("My router", vanguard.router(), "hosts", map.hosts)
for _, host in ipairs(map.hosts) do
    local scan = {action=vanguard.sense.HOST_SCAN, target=host.id}
    if vanguard.can_deploy(scan) then
        local result = vanguard.await(vanguard.deploy(scan))
        print(host.id, result.data.assessment)
    end
end
```

`deploy` starts the job. `await` waits for it. `.data` holds the findings. Get comfortable with those three; most of your first agent will be variations on this loop.

Your main code can return. Registered handlers and admitted jobs keep working, so you don’t need a loop whose only job is to keep Lua breathing. For the opposite of a defence plan, run `vanguard/agents/ostrich.lua`. It does nothing. That is the entire agent.

Packet has [the map briefing](/docs/world/). Read it before you start pulling plugs. Scoreboard is quite particular about servers remaining useful.
