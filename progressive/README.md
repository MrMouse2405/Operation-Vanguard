# Progressive networks

Ten complete `network.toml` files, from a single small LAN to a multi-site
10,000-node challenge. **Total nodes includes routers, workstations and servers.**
Each LAN has at most **500 total nodes**, including its router and servers.

| Level | Network | Total nodes | LANs | Servers | Initial infections | Deadline |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 01 | [10 nodes](01-10-nodes/network.toml) | 10 | 1 | 1 | 1 | 120 |
| 02 | [100 nodes](02-100-nodes/network.toml) | 100 | 2 | 2 | 2 | 180 |
| 03 | [250 nodes](03-250-nodes/network.toml) | 250 | 3 | 6 | 4 | 240 |
| 04 | [500 nodes](04-500-nodes/network.toml) | 500 | 5 | 10 | 8 | 300 |
| 05 | [1,000 nodes](05-1000-nodes/network.toml) | 1,000 | 8 | 16 | 16 | 360 |
| 06 | [1,500 nodes](06-1500-nodes/network.toml) | 1,500 | 12 | 24 | 30 | 420 |
| 07 | [2,500 nodes](07-2500-nodes/network.toml) | 2,500 | 16 | 32 | 48 | 480 |
| 08 | [5,000 nodes](08-5000-nodes/network.toml) | 5,000 | 24 | 48 | 96 | 540 |
| 09 | [7,500 nodes](09-7500-nodes/network.toml) | 7,500 | 32 | 64 | 160 | 600 |
| 10 | [10,000 nodes](10-10000-nodes/network.toml) | 10,000 | 40 | 80 | 240 | 660 |

## Run a level

```sh
vanguard --script agent.lua --network progressive/01-10-nodes/network.toml
vanguard --script agent.lua --network progressive/06-1500-nodes/network.toml
vanguard --script agent.lua --network progressive/10-10000-nodes/network.toml
```

For a full scored attempt at level 10:

```sh
vanguard --script agent.lua --network progressive/10-10000-nodes/network.toml --headless --ticks 660 --replay level-10.json
```

Use each file's published deadline for a full attempt. Verified recovery stops a
run early; a shorter `--ticks` limit is just a partial test.

## How the progression works

- **01:** local discovery, containment and recovery on one LAN.
- **02–03:** cross-LAN required services and a simple router chain.
- **04–05:** rings/chords, uneven LAN workloads and more simultaneous incidents.
- **06:** three sites with four LANs each; transit decisions affect other LANs.
- **07–08:** four sites, three service dependencies per workstation, and multiple
  initial sources on some LANs. Work must be prioritized under the task-slot limit.
- **09–10:** more simultaneous sources, larger traffic histories, more competing
  investigations, and additional WAN paths. Level 10 starts with 240 infections,
  compared with eight in the separate `final-test/` fixture.

The files increase scale, infection pressure and dependency complexity together.
Average initial sources per LAN rise from one to six, so adding routers does not
cancel out the increased incident load through extra parallel task slots.
More LANs also provide more private VMs/task slots, so node count alone is not a
strict difficulty measure for every policy. Longer deadlines allow recovery work;
points still reward preserving service and controlling infection early.

All levels use standard fuel, memory, task-slot and infection settings. Mission
defaults stay at 90% weighted service, 14 consecutive recovery ticks, and a
30-tick total service-outage defeat window. Every server has required clients;
no orphan critical servers or hidden scripted infection waves are present.

IDs are non-contiguous; use discovery instead of assuming LAN membership from IDs.
Initial infections occupy mixed positions in local ID order. Required-service
pairs are explicit: local service has weight 2, within-site/adjacent-LAN service
has weight 2, and levels 07–10 add remote-site service with weight 3.

## Validation and initial calibration

All ten files pass the engine's topology/mission validator and independent
connectivity, node-count, service-reference and LAN-cap checks. Regeneration is
byte-identical. The earlier traffic-prioritized organizer control was run through
the full deadline on the first six levels, using the normal Lua API:

| Level | Control result | Tick | Score |
| --- | --- | ---: | ---: |
| 01 | VICTORY | 40 | 977.96 |
| 02 | VICTORY | 72 | 990.44 |
| 03 | VICTORY | 45 | 998.26 |
| 04 | VICTORY | 45 | 998.57 |
| 05 | VICTORY | 89 | 997.53 |
| 06 | DEFEAT | 278 | 198.74 |

All six runs had zero Lua runtime errors. Levels 07–10 have structural/engine
validation, not full terminal control benchmarks. The control is an initial
calibration point, not a required competitor solution or a universal difficulty
ranking. Successful play at higher incident loads needs better scheduling,
containment and recovery decisions than that simple control demonstrated.

## Regenerate

```sh
python3 progressive/generate.py
```

The standard-library-only generator reproducibly writes all ten complete files.
The separate final test and default small topology are independent of this suite.
