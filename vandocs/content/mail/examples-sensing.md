This is a **complete read-only agent**. It changes no shields or power settings. Save the attachment as `agent.lua`, or use the supplied file:

```sh
vanguard --script vanguard/agents/examples/sensing.lua --network vanguard/networks/tutorial-1.toml
```

<!-- attachment:agents/examples/sensing.lua -->

## Read the output in this order

1. **SCAN** lists local hosts and adjacent routers. It discovers IDs; it does not assess infection.
2. **SHIELD_STATUS** without target describes your router’s rules.
3. **TRAFFIC_SCAN** lists historical endpoint pairs that used this router. Same-LAN traffic is absent.
4. **HOST_SCAN** gives a quick assessment. **CONNECTION_SCAN** lists current peers and directions.
5. **FULL_HOST_SCAN** gives authoritative compromise at its completion tick plus connections.
6. **SHIELD_STATUS** with target reads that local host’s policy, exceptions and isolation.

The helper uses `await_any` and checks `is_ready` so a scan failing mid-flight does not crash the whole example. An offline host cannot answer a health scan. It can still have its shield read/configured.

The program finishes after its circuit. Existing handlers and jobs would survive a main return, but this program has neither left to do. A read-only agent can lose the match: observations are not treatment. [All result shapes and durations](/docs/actions/scans/) are in the scan reference.
