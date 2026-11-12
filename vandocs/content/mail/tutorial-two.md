Now there are **two routers, twenty workstations and two servers**: twenty-four total nodes, ten workstations and one server per LAN. Every workstation needs both servers under the default mission. Blocking the cross-LAN route can therefore lose real service.

## Minutes 15–20: same program, two notebooks

```sh
vanguard --script vanguard/agents/coordinated-defender.lua --network vanguard/networks/tutorial-2.toml --port 0
```

Each router runs the same file with private memory. Open telemetry and use the **Filter Vanguard agents** selector. A scan from router 1 does not populate router 2’s local variable. [Identity](/docs/api/identity/) gives you the agent-to-router map; do not guess agent IDs from router numbers.

## Minutes 20–25: send an early warning

```sh
vanguard --script vanguard/agents/examples/early-warning.lua --network vanguard/networks/tutorial-2.toml --port 0
```

Read [the complete warning attachment](/docs/examples/warning/), then watch **Pub / Sub**. The source router confirms infection and reports its host ID. A receiver uses TRAFFIC_SCAN to learn endpoints seen in transit, then can block that known source at its router. It cannot restore the remote workstation. This example contains; it deliberately does not implement recovery.

If the attack remains inside one LAN, there may be no transit history to discover. No history means no evidence of that cross-LAN connection. A warning alone does not create discovery. Use [the three-LAN sample](/docs/networks/small/) and more seeds to explore different routes.

## Minutes 25–30: appoint someone who enjoys paperwork

```sh
vanguard --script vanguard/agents/examples/leader.lua --network vanguard/networks/tutorial-2.toml --port 0
```

[The leader example](/docs/examples/leader/) elects the lowest agent ID, collects reports, and tells the reporting owner to isolate its host. Leadership adds at least a message round-trip. A local emergency may be better handled locally before asking headquarters.

## The complete second network

<!-- attachment:networks/tutorial-2.toml -->

## Your next experiment

Compare solo, coordinated and leader approaches on the **same** file and seed. Which reacts earliest? Which actually restores service? A containment-only example may lose despite stopping spread. Keep one measurement per run, then change one decision. That is enough process to get you through the next six and a half hours.
