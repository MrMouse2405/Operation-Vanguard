Every router can compute the lowest active **agent ID** from `self()` and `others()`. They agree on a leader without a preloaded router-to-agent arithmetic trick.

```sh
vanguard --script vanguard/agents/examples/leader.lua --network vanguard/networks/tutorial-2.toml
```

<!-- attachment:agents/examples/leader.lua -->

## Follow the report

A worker scans its own host, sends a report to the leader, and receives an isolation order. The worker checks that the sender is the leader and the target belongs to its local discovery. Its main loop executes the order; handlers only record it.

The leader’s own LAN is handled locally: **self-send is not allowed**. This sample uses a fixed startup election; it does not implement leader replacement or a distributed consensus protocol. We have seven hours, not a dissertation defence.

## What the example is missing on purpose

It demonstrates command flow and ownership, not recovery. A useful next design would let owners contain emergencies immediately, send evidence to the leader, and ask the leader to coordinate recovery windows. Centralizing every trivial scan decision adds messages and fuel while local task slots remain distributed.

Compare this with [local patrols](/docs/examples/solo/) and [peer warnings](/docs/examples/warning/) before deciding whether your network needs headquarters.
