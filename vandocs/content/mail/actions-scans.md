If you tell me a host is clean, I’m going to ask which scan, and at what tick. A scan result is evidence with a timestamp. Keep both.

Scans use a task slot without locking the target. They can run alongside a mutation, which means the host may change before the scan finishes.

Start one with `vanguard.deploy({action=vanguard.sense.ACTION, target=host_id})` when a target is needed. Use `await` to get its result. The shapes below are the result’s **`.data`**. `observed_at` is the tick it finished, and returned lists are sorted.

## SCAN — 2 ticks

No target or peer. Finds your local hosts and adjacent router IDs. It doesn’t inspect neighbouring routers. Hosts stay in the list even if they’re off or blocked.

```lua
{observed_at=tick, hosts={{id=1001,type="WORKSTATION"}}, routers={2,4}}
```

## TRAFFIC_SCAN — 4 ticks

No target, peer, window, or cursor. Shows every unique successful pair of endpoints that has used your router since the run began. Each pair has `a < b`; the list sorts by `a`, then `b`. Those endpoint IDs become known.

Blocked attempts and same-LAN traffic aren’t included. Repeat traffic adds no duplicates. Closed connections, shutdowns, and restores don’t erase history. I use this to ask “who has talked through here?” **It does not tell me whether the connection still works, or whether either endpoint is infected.**

```lua
{observed_at=tick, pairs={{a=1001,b=1005}}}
```

## HOST_SCAN — 3 ticks

Needs a discovered, local, online host. A quick symptom check: normally `CLEAN`, `SUSPICIOUS` for exposure, or `COMPROMISED` for established infection. Write down when it finished. “CLEAN several attacks ago” is an old observation, not a current assessment.

```lua
{observed_at=tick, power="ONLINE", assessment="SUSPICIOUS"}
```

## CONNECTION_SCAN — 3 ticks

Needs a discovered, local, online host. Shows its current connections, their direction relative to that host, and when they opened. Multiple connections to the same peer are possible. Sorted by peer, opening tick, then direction.

```lua
{observed_at=tick, connections={{peer=1005,direction="INBOUND",opened_at=1}}}
```

You may find a remote peer here. You have learned its ID, not acquired jurisdiction. Send the evidence to its owner.

## FULL_HOST_SCAN — 8 ticks

Needs a discovered, local, online host. Returns an exact infection boolean and its current connections, all from the same finish-time snapshot. Connections use the same format as `CONNECTION_SCAN`.

```lua
{observed_at=tick, power="ONLINE", compromised=false, connections={}}
```

## SHIELD_STATUS — 1 tick

No target checks your router; a discovered local host target checks that host. Another router isn’t a valid host target. Works even when a host is off.

```lua
{observed_at=tick, policy="DEFAULT_ALLOW", exceptions={1005}, isolated=false}
```

For DEFAULT_ALLOW, exceptions are blocked peers. For DEFAULT_DENY, they’re allowed peers. Routers always report `isolated=false`.

Host scans can fail with `UNKNOWN_NODE`, `UNDISCOVERED`, `OUT_OF_SCOPE`, `INVALID_TARGET_TYPE`, `OFFLINE`, or `TARGET_BUSY`. The engine checks at the start and finish. Read a later failure with `status`; `result` and `await` raise it.

Pick the scan that answers your question. Eight ticks of FULL_HOST_SCAN may buy certainty you need. Or it may be five ticks you should have spent isolating an obvious problem.
