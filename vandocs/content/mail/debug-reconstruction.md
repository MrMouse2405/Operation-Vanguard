One view says what discovery learned. The second adds health assessments as scans finish. Both are based on the agent’s own evidence. No peeking at the dashboard’s infection count.

```sh
vanguard --script vanguard/agents/examples/debug-map.lua --network vanguard/networks/tutorial-2.toml --dbg
```

<!-- attachment:agents/examples/debug-map.lua -->

## Compare the panes

1. Start or step the run past discovery and a few health scans.
2. Open **Compare agent drawings · dbgguard**.
3. On the left, select one agent’s **discovery** view.
4. On the right, select that agent’s **assessments** view.
5. Expand node info and compare `observed_at` with the current tick.

Discovery knows membership, not health. The assessment view begins UNKNOWN and replaces info when a host scan succeeds. The first view deliberately remains unchanged. If the host changes later, that old assessment may be stale; your drawing should say when it was observed.

## Reconstruct connections as a separate exercise

Use CONNECTION_SCAN on a discovered local host. For each returned `peer`, add or update a peer node before adding the edge. Put `direction` and `opened_at` in edge info; the scan discovers those peer IDs, but it does not tell you every remote host’s LAN or health. Label unknowns as unknowns.

Here is the complete connection exercise. It clears each circuit, keeps observation ticks and draws peers as UNKNOWN rather than guessing their LAN or health. Scans finish at different ticks, so even one circuit is not an atomic snapshot.

<!-- attachment:agents/examples/debug-connections.lua -->

For a historical view, keep edges and label observation ticks instead of clearing. Do not label historical traffic “current connections.” [Scanner’s six scans](/docs/actions/scans/) answer different questions.

This agent only observes, so it is expected to lose infected missions. Debug drawings help explain a policy; they are not the policy itself.
