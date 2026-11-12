This attachment is **Network Agent.md**. It is an instruction sheet for a topology-generating LLM, not a Lua defender.

Download it below, or find it under `vanguard/networks/` after extracting **Download example files** from the toolbar. Its full text is embedded here even when you open the single-file handbook offline.

Copy it into a fresh conversation, then ask for a network shape and counts. You may supply public documentation and sample TOML. **Do not supply your competition agent code.** The authorization covers network data, research and learning, not generated competition agent logic.

## A request that is specific enough

```text
Using the Network Agent instructions below, make a 24-node network:
2 routers, 20 workstations, 2 servers. Use one LAN per router,
one server per LAN, a router link, seed 7 and a 300-tick horizon.
Infect one workstation in LAN 1. Every workstation needs both servers.
Output the complete TOML and a count summary. No Lua or agent code.
```

## Copy these instructions

<!-- attachment:networks/Network Agent.md -->

## Validate the answer, not the confidence

Save only the TOML in `practice.toml`; keep prose and Markdown fences out of the file. Then:

```sh
vanguard --script vanguard/agents/ostrich.lua --network practice.toml --print-config
```

If validation fails, send the public TOML and parser error back for correction, not your agent. Verify node counts and mission pairs yourself. For a very large network, [the supplied generator](/docs/networks/scale/) is more reliable than asking a model to count thousands of lines.
