The smallest useful allowlist: one local workstation may talk only to one local server. This example uses the first discovered workstation and server, not hard-coded IDs.

```sh
vanguard --script vanguard/agents/examples/shields.lua --network vanguard/networks/tutorial-1.toml
```

<!-- attachment:agents/examples/shields.lua -->

## What survives isolation?

After DEFAULT_DENY, the workstation blocks everyone. ALLOW_PEER_ON_HOST adds the server as an exception. ISOLATE_HOST then overrides all data traffic without deleting that exception. RELEASE_HOST brings the saved allowlist back into use.

Each action takes one tick. Changing policy clears exceptions; repeating the same policy preserves them. Router-only versions omit `target`, even when you know your own router ID. [Every shield action](/docs/actions/shields/) has its fields listed.

This policy is **not a cure**. An infected workstation allowed to reach a server can still attack that server. It is also not a complete service policy when the mission requires remote servers. Read the required pairs before deciding which doors are unnecessary.
