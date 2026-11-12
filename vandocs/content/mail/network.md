I use two-router networks for experiments. Small enough to know where every cable goes; large enough to catch a rule that only works inside one LAN. Save this as a TOML file:

```toml
version = 1
routers = [{ id = 1 }, { id = 2 }]
lans = [{ id = 10, router = 1 }, { id = 20, router = 2 }]
workstations = [
  { id = 1001, lan = 10, compromised = true },
  { id = 1002, lan = 20 },
]
servers = [{ id = 2001, lan = 20 }]
connections = [{ from = 1, to = 2 }]

[simulation]
seed = 1
tick_limit = 1000
tick_ms = 0
```

Each router owns **one LAN**. Every host belongs to one LAN. `connections` joins routers in both directions; hosts are attached automatically through LAN membership. All routers must be reachable from one another.

Node IDs must be unique across routers, workstations, and servers. LAN IDs are a separate set of names, so a LAN and a node may share a number.

Only workstations accept `compromised = true` at startup. Leave it out for a clean host. There’s no secret starting infection. The same supplied script runs on every router; you don’t choose scripts in TOML.

## Give the servers a job

A finite network with workstations and servers is scored by default. Normally every workstation needs to reach every server. To choose specific required connections:

```toml
[mission]
recovery_ticks = 14
outage_ticks = 30
service_threshold_percent = 90
services = [{ client = 1001, server = 2001, weight = 1 }]
```

Use `enabled = false` under `[mission]` for practice. `tick_limit = 0` means unlimited practice. [Here’s how scoring works.](/docs/scoring/)

## Settings I check before a run

`tick_ms` is the minimum turn length in milliseconds, including computation. **0 means full speed**. You can change it live with the dashboard’s **s/tick** control. A slow turn never gets an extra full delay. Speed doesn’t change game results.

Other optional simulation settings are `fuel_per_tick`, `instruction_quantum`, `memory_limit`, `max_payload_bytes`, `max_handlers`, `native_call_cost`, and `task_admission_cost`. They limit how much code, memory, and messaging an agent can use. `infection_intervals` and `attack_slots` control infection timing and concurrent attacks.

The dashboard’s scenario information shows the actual settings. Check there before blaming the file you meant to load. When comparing agents, keep the network, seed, and budgets the same. Otherwise you’ve changed the experiment halfway through.
