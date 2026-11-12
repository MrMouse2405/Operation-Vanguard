Four baseline entrypoints live in `vanguard/agents/`. They are organizer-supplied examples: you may study, copy and adapt them. Read the implementation before treating a filename as a performance guarantee.

| Agent | Entrypoint | Policy |
| --- | --- | --- |
| Ostrich | `ostrich.lua` | No defence. The control experiment. |
| Basic Defender | `basic-defender.lua` | Local rolling scans; five concurrent slots; isolate, restore, release |
| Coordinated Defender | `coordinated-defender.lua` | Local patrol plus shared confirmed-source reports and transit containment |
| Advanced Defender | `advanced-defender.lua` | Server-first patrol and priority response to usable warnings, plus coordination |

## A benchmark is a network and a command

The [scale sample](/docs/networks/scale/) generates exactly **1,000** or **10,000 total nodes** with explicit required local services. Start with Basic at 1,000 and Coordinated at 10,000. Advanced has no hard-coded topology size or host IDs; it uses the same API limits on each router.

```sh
vanguard --script vanguard/agents/basic-defender.lua --network sample-1000.toml --headless --ticks 300 --replay basic.json
vanguard --script vanguard/agents/coordinated-defender.lua --network sample-10000.toml --headless --ticks 300 --replay coordinated.json
vanguard --script vanguard/agents/advanced-defender.lua --network sample-10000.toml --headless --ticks 300 --replay advanced.json
```

**No agent can promise victory on every legal network.** A short horizon, many initial infections, a crowded single LAN or stricter required pairs can make the same policy lose. Ostrich loses infected evaluation fixtures; an entirely clean network may let a do-nothing agent win. Count outcomes and service loss, not just “no Lua errors.”

### Published sample results

The supplied generator, seed **1**, default budgets, **300 ticks**, one initially compromised workstation, local-server required services, debug disabled:

| Agent | Total nodes | Result | Points / 1,000 | Lua runtime errors |
| --- | --- | --- | --- | --- |
| Basic | 1,000 (50 routers) | VICTORY | 999.91 | 0 |
| Coordinated | 10,000 (100 routers) | VICTORY | 999.97 | 0 |
| Advanced | 10,000 (100 routers) | VICTORY | 999.97 | 0 |
| Ostrich | 1,000 (50 routers) | DEFEAT at tick 127 | 214.52 | 0 |

Ostrich also loses both tutorial networks at tick 57. These are reproducible examples, not hidden test-suite results. Increase initial infection and change the topology to make a more demanding evaluation.

## Where the work happens

The three defenders share `patrol.lua` beside their entrypoints. Copy that module as well if moving an agent. Each host has one state machine: scan → isolate → restore → release. Jobs are overlapped across hosts, while mutations on one host remain serialized. Failed jobs do not get labelled successful.

Basic does not message. Coordinated publishes confirmed threats; receivers discover transit peers before applying router blocks and remove those blocks after a reported cure. Advanced additionally scans servers first and prioritizes useful local warnings. None sees dashboard truth.

The immediate-release policy remains a baseline, not perfect recovery reasoning. Re-infection is possible while another source remains. Improve that decision yourself and compare the same seeds. Fuel and five task slots still apply at ten thousand nodes.
