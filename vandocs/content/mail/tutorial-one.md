This exercise has **one router, ten workstations and one server**: twelve total nodes. “Ten nodes” in the classroom shorthand means ten client machines here; routers and servers count separately in the file.

You need the supplied `vanguard` executable, the exercise files, a text editor and a browser. Start in the exercise folder. If `vanguard` is not found, ask the organizers to check your setup before spending competition time on it. The dashboard is included.

## Minutes 0–5: launch

```sh
vanguard --script vanguard/agents/examples/sensing.lua --network vanguard/networks/tutorial-1.toml
```

Open the URL printed in the terminal. Default: **http://127.0.0.1:8000**. If the port is taken, add `--port 0` and open the assigned URL. Press **+1** twice, then open telemetry **Output**. Discovery finishes at tick 2; the agent reads and prints it when it resumes on the next step. Advance again if needed.

See eleven local hosts: ten WORKSTATIONs and SERVER 2001. Workstation 1001 starts compromised. The router is immune.

## Minutes 5–10: inspect without confusing yourself

Search for **1001**, select it, and compare **True state** against **Discovered evidence**. You, the operator, may see infection before your Lua code has scanned it. That difference is intentional.

The full [interface tour](/docs/interface/start/) is available if a button is hiding. The [sensing attachment](/docs/examples/sensing/) explains the complete script you just ran.

## Minutes 10–15: run a patrol

```sh
vanguard --script vanguard/agents/examples/solo.lua --network vanguard/networks/tutorial-1.toml --port 0
```

The [solo attachment](/docs/examples/solo/) discovers hosts, checks them, isolates suspicious or compromised ones, restores them, and releases them. It uses one task at a time. Watch the **Jobs** tab: a restore takes twenty ticks even when your code is short.

Copy that attachment into your own `agent.lua` to experiment. Supplied examples are allowed; write your own changes. Try changing the print message first. **Restart** rereads the file. Then try delaying release and observe its cost to service. Keep a working copy before editing.

## The network, in full

Save the block as `practice.toml` if you want a separate copy:

<!-- attachment:networks/tutorial-1.toml -->

## Checkpoint

You should be able to name the infected workstation, explain why shutdown is not a cure, and find a task’s start and finish tick. Try `basic-defender.lua` next: it overlaps jobs instead of waiting on every host in sequence.

```sh
vanguard --script vanguard/agents/basic-defender.lua --network vanguard/networks/tutorial-1.toml --headless --ticks 300 --replay tutorial.json
```

Inspect `snapshot.score` and the agents’ `runtime_errors` in `tutorial.json`. If your script failed, read [Debug’s error desk](/docs/errors/). If your script ran, move to [Tutorial 2](/docs/tutorial/two/).
