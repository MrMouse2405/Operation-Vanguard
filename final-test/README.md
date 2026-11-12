# Final test: Four Sites, One Bad Morning

Use **network.toml** in this folder. It is a complete explicit topology, not a
generator directive, and works with the installed `vanguard` executable:

```sh
vanguard --script agent.lua --network final-test/network.toml
```

For a final scored replay:

```sh
vanguard --script agent.lua --network final-test/network.toml --headless --ticks 240 --replay final-run.json
```

## Published mission

| Setting | Value |
| --- | ---: |
| Total nodes | **10,000** |
| Routers / LANs | 40 / 40 |
| Workstations | 9,880 |
| Servers | 80 |
| Maximum nodes per LAN, including router and servers | **500** |
| Initially compromised workstations | 8 |
| Router links | 54, undirected |
| Required client/server pairs | 29,640 |
| Deadline | 240 ticks |
| Recovery verification | 14 consecutive ticks |
| Required-service recovery threshold | 90% of weighted pairs |
| Total service-outage defeat window | 30 consecutive ticks |
| Seed | 20261112 |

All other settings use the standard engine defaults, including fuel, memory,
five task slots, two attack slots and fourteen-tick infection. No hidden waves,
special host rules, timed surprises or private agent capabilities are used.
Verified full recovery ends the run early under the current engine rules.

## Why this is harder than the scale sample

- **Uneven workloads:** eight campus LANs have 497 workstations each; twelve
  regional LANs have 348 or 248; twenty branches have 148, 98 or 94. Each LAN
  also has two servers and one router, bringing its total to at most **500 nodes**.
  A policy must handle the same five slots under very different loads.
- **Four connected sites:** site rings and chords feed a WAN ring, with two
  additional regional WAN links. Transit filters can affect several LANs' service.
  Routes remain the engine's deterministic shortest paths; blocked paths do not
  automatically reroute.
- **Distributed infection:** eight sources span campus, regional and branch LANs.
  Their positions include the start, middle and end of local sorted-ID lists.
  Ascending and descending health scans have different blind spots.
- **Real service dependencies:** every workstation needs one local service
  (weight 2), another LAN's service within its site (weight 2), and a different
  site's service (weight 3). **71.43% of service weight crosses LAN boundaries.**
  The file explicitly specifies these pairs; it does not expand to every server.
- **No arithmetic topology shortcut:** node, router and LAN IDs are positive,
  unique and non-contiguous. Discovery is the reliable source of membership.
- **Blanket containment is insufficient:** shutting down or isolating everything
  loses service; leaving infected sources isolated still prevents victory.

The key discriminator is choosing useful evidence and actions before a full
health-scan circuit finishes. It is not a contest to submit the most API calls.

## Feasibility and calibration

The real release engine accepted the file. An organizer validation agent using
ordinary local discovery, traffic-history leads, host scans and recovery achieved
**VICTORY at tick 45**, **999.91 / 1,000**, with **zero Lua runtime errors**.
It used the same source on all routers, no topology-specific IDs, no private state
and no altered engine budgets. The validation solution is kept outside this
distributable final-test folder.
Two complete runs produced the same **46 semantic hashes** (ticks 0 through 45).
The three supplied defenders also completed 12-tick smoke runs without Lua errors;
each still had eight compromised hosts at that point.

The supplied Basic, Coordinated and Advanced rolling patrols cannot complete
their ordered campus sweeps before this deadline. Even ignoring all discovery,
traffic scans, isolation, restoration and release work, five slots permit at
most **400 three-tick host scans per router by tick 240**. Four initial sources
occupy position **497** in those campus workstation lists. A short smoke run
is not a full terminal benchmark; this deadline bound explains the limitation.

This demonstrates a useful baseline discriminator and a valid winning policy;
it cannot prove that every conceivable weak algorithm loses. Rank victories
before incomplete/defeat, then compare points under the published scoring rules.
Use the same network, seed and engine revision for every competitor. Large
outbreaks require more wall-clock computation than controlled runs; `tick_ms=0`
removes intentional pacing, not the engine's computation time.

## Reproduce or edit

```sh
python3 final-test/generate.py
```

This standard-library-only generator deterministically recreates the complete
file. Change the generator, then regenerate, rather than hand-editing 29,640
service entries. The final-test topology is kept separate from the small default
`network.toml` and the handbook's tutorial/sample downloads.
