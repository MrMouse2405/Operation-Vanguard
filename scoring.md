# Scoring

Keep services running. Remove the botnet. Bring quarantined machines back.

## Outcome

- **Victory:** no infected hosts or active exposure; all servers provide service;
  at least 90% of required connections work, for 14 consecutive ticks through the deadline.
- **Incomplete:** reach the deadline without verified recovery.
- **Defeat:** no required server connection works for 30 consecutive ticks.

Offline or isolated infection still counts. Containment is not eradication.
Recovery can be lost again. Matches continue to the fixed deadline after recovery.

## Score: 0–1,000

| Points | Measure |
| --- | --- |
| 600 | Average fraction of required service connections working each tick |
| 250 | Average fraction of hosts free of established infection each tick |
| 150 | Fraction of hosts clean, online, released, and able to fulfil their required connections at the end |

A connection works when both hosts are clean, online, released, and their route
permits traffic. Exposure alone does not remove credit. Routers are not hosts.
Hosts have equal security/recovery weight. Connections have published weights.

By default every workstation must reach every server. Scenarios may specify a
smaller service list. All servers are critical and must have a required client.
`scenario.mission` publishes the rules; `score` gives the live breakdown.
Live points use the full deadline; recovery points are provisional until the end.
Early defeat earns zero for remaining ticks and zero final-recovery points.

Rank **victory first, incomplete second, defeat last**, then points within each.
For a suite: victory rate, then mean per-scenario points. Everyone uses identical
networks, seeds, deadlines and resource limits. Fuel is a budget, not a bonus.
Pausing and browser speed never affect points.

## Examples

- Perfect clean service for the whole match: **1,000**.
- Shut everything down: little service credit; infected hosts stay infected.
- Isolate every host: prevents spread but sacrifices service and recovery points.
- Quarantine → restore → release: earns more when done early and selectively.

## Scenario options

```toml
[mission]
recovery_ticks = 14
outage_ticks = 30
service_threshold_percent = 90
services = [
  { client = 1001, server = 1017, weight = 1 },
  { client = 1002, server = 1018, weight = 1 },
]
```

Omit `services` to require every workstation/server pair. Omit `[mission]` for
these defaults in finite networks with workstations and servers. Set
`enabled = false` under `[mission]` for practice. Unlimited or server-free
networks default to unscored practice.
