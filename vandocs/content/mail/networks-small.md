Copy an entire TOML block into a file. The CLI chooses the agent separately. Comments begin with `#`, and every list can be written over several lines.

## One LAN: the first tutorial

Ten workstations, one server, one router. One workstation starts infected.

<!-- attachment:networks/tutorial-1.toml -->

## Two LANs: the second tutorial

Twenty workstations, two servers, two routers. [Copy the complete two-LAN file from Tutorial 2](/docs/tutorial/two/). Every workstation requires both servers by default.

## Three LANs: traffic through the middle

Three routers in a line. Router 2 can observe successful transit between LAN 1 and LAN 3, but its history does not contain same-LAN traffic.

<!-- attachment:networks/three-lan.toml -->

## Rules for writing your own

1. Set `version = 1`.
2. Give every router, workstation and server a unique positive node ID.
3. Give each router exactly one LAN. LAN IDs are a separate namespace.
4. Attach hosts using `lan`. Do not write host-to-router cables in `connections`.
5. Connect all routers with undirected `from` / `to` edges. No self-links or duplicate reversed links.
6. Mark initial infection only on workstations with `compromised = true`. Omitted means clean. Routers are immune; servers start clean.
7. Choose the seed, horizon and required services. Only use `enabled = false` under `[mission]` for deliberately unscored practice.

```sh
vanguard --script vanguard/agents/ostrich.lua --network practice.toml --print-config
```

This validates the file and prints effective settings without starting. A typo is rejected rather than silently inventing a topology. [Network basics](/docs/network/) lists optional mission fields; [larger samples](/docs/networks/scale/) use a generator instead of hand-editing ten thousand rows.
