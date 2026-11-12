One router minds its own LAN. No topics, no leader, no committee. This is easy to follow and intentionally serial: a twenty-tick restore delays the next host’s scan.

```sh
vanguard --script vanguard/agents/examples/solo.lua --network vanguard/networks/tutorial-1.toml
```

<!-- attachment:agents/examples/solo.lua -->

## Trace one infected machine

Discovery learns the host ID. HOST_SCAN finds symptoms. ISOLATE_HOST cuts traffic. RESTORE cures the host. RELEASE_HOST returns it to saved rules. Each stage waits for completion before continuing.

There is a deliberate policy weakness: release immediately after restoring, even if another attacker remains. You can get a clean machine reinfected. What evidence would make release safer? What service do you lose by waiting? Write your own decision and measure it.

On a larger LAN the serial circuit grows slow. [The Basic Defender](/docs/examples/defenders/) overlaps five jobs through state machines. Do not start five mutations against the same host; they share a lock, not a queue.
