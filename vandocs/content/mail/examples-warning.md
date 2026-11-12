An infected machine on router A’s LAN can attack a machine beyond router B. Router A can confirm the source and warn teammates. Router B can block that original endpoint at its own perimeter **after learning its ID through a scan**.

This is a complete **containment example**, not a full defender. It leaves quarantine and transit blocks in place. Add measured recovery and unblock decisions in your own policy.

```sh
vanguard --script vanguard/agents/examples/early-warning.lua --network vanguard/networks/tutorial-2.toml
```

<!-- attachment:agents/examples/early-warning.lua -->

## Why all these steps?

- Subscribe first, then wait a tick for membership to commit.
- Confirm established infection with HOST_SCAN. CONNECTION_SCAN supplies current peers and directions; not every connection by itself proves an attack.
- Publish the source host and observation tick. Other routers have independent memory.
- At a receiver, TRAFFIC_SCAN can discover remote endpoints that traversed its router. A message containing `1001` alone does **not** satisfy discovery.
- BLOCK_PEER has no target field. It blocks that endpoint’s transit traffic; it does not stop two hosts talking inside one LAN.

An attempted but blocked connection does not enter traffic history. If the source never traversed the receiving router, `can_deploy` can return UNDISCOVERED, and this example keeps the report for later. That is preferable to pretending a report grants omniscience.

For a repeatable cross-LAN demonstration, use a two-router network with one workstation on LAN 1 and one server on LAN 2. The only clean target is remote. You can build it from [the network schema](/docs/network/). In Tutorial 2, target selection depends on the seed; the source may attack locally instead.

The next attachment uses [a leader](/docs/examples/leader/). Leadership does not remove message latency either.
