Do not type ten thousand hosts by hand. That is how a network ends up with workstation 1007 attached to the kettle.

The supplied `vanguard/networks/generate.py` uses **Python 3’s standard library** to write explicit version-1 TOML. It creates a star of routers, one server per LAN, distributes workstations across LANs, infects the first workstation, and publishes **each workstation’s local server** as its required service. That mission differs from the default all-workstation/all-server pairs; compare agents on the same mission.

## Three useful sizes

```sh
python3 vanguard/networks/generate.py --nodes 100 --routers 5 --output sample-100.toml
python3 vanguard/networks/generate.py --nodes 1000 --routers 50 --output sample-1000.toml
python3 vanguard/networks/generate.py --nodes 10000 --routers 100 --output sample-10000.toml
```

`--nodes` counts **routers + workstations + servers**. At 1,000: 50 routers, 50 servers, 900 workstations. At 10,000: 100 routers, 100 servers, 9,800 workstations. The engine supports at most 1,000 routers; the generator requires at least one workstation and server per LAN. Large runs take real wall-clock time; deterministic does not mean instantaneous.

Use `--seed 7` to vary enemy selection, and `--ticks 1000` for a longer horizon. Keep both fixed when comparing policies. Validation:

```sh
vanguard --script vanguard/agents/basic-defender.lua --network sample-1000.toml --print-config
```

## The generator, for inspection or copying

<!-- attachment:networks/generate.py -->

## Pick more than one shape

The star keeps routes short. Change the connections to a line or ring to test transit filters, increase initial infection for simultaneous incidents, and compare large LANs with many small LANs at the same total node count. Five slots per router make those layouts very different workloads.

Want an LLM to write topology data? [Network Agent.md](/docs/networks/agent/) provides the constraints. Keep your competition Lua out of that conversation.
