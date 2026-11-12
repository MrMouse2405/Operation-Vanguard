Tell me who needs to talk to whom. “Block everyone” is a short policy, but Scoreboard will have questions about the server you made unreachable.

**Every action here takes 1 tick**, a task slot, and the target’s mutation lock. Success contains `data={changed=boolean}`.

Use `action=vanguard.shield.ACTION`. Router actions **omit target**. Host actions need a discovered local host as `target`. You can configure an offline host too.

## At the router

| Action | Fields to supply | What it does |
| --- | --- | --- |
| `SET_DEFAULT_ALLOW` | action | Allow traffic except blocked endpoints |
| `SET_DEFAULT_DENY` | action | Block traffic unless both original endpoints are allowed |
| `BLOCK_PEER` | action, peer | Add a blocked exception, or remove an allowed one |
| `ALLOW_PEER` | action, peer | Remove a blocked exception, or add an allowed one |

## At the host

| Action | Fields to supply | What it does |
| --- | --- | --- |
| `SET_DEFAULT_ALLOW_ON_HOST` | action, target | Allow everyone except blocked peers |
| `SET_DEFAULT_DENY_ON_HOST` | action, target | Allow only peers on the exception list |
| `BLOCK_PEER_ON_HOST` | action, target, peer | Block that peer |
| `ALLOW_PEER_ON_HOST` | action, target, peer | Allow that peer |
| `ISOLATE_HOST` | action, target | Stop all data traffic, keeping saved rules |
| `RELEASE_HOST` | action, target | Remove isolation and use saved rules again |

Peer actions need a known node ID different from the target. Other actions reject `peer`.

## Read these before you deploy the rule

- Hosts and routers start at DEFAULT_ALLOW with no exceptions or isolation.
- **Changing** policy clears exceptions. Repeating the **same** policy keeps them.
- Isolation overrides a host’s rules without deleting them. Restore also preserves rules and isolation.
- Host rules check the other endpoint. A DEFAULT_ALLOW router blocks if **either** original endpoint is blocked. DEFAULT_DENY needs **both** original endpoints allowed.
- Intermediate router IDs don’t replace those original host IDs.
- Rules apply in both directions, to new and existing connections. Newly blocked connections close. Allowing traffic doesn’t reopen old ones.
- Same-LAN traffic skips router rules, but still checks host rules.
- You can’t isolate a router. Management and teammate messages are separate from host data traffic.
- Releasing a host doesn’t power it on or erase its own rules.

```lua
-- known_peer_id must already be discovered; a remote peer is fine.
local rule = {action=vanguard.shield.BLOCK_PEER, peer=known_peer_id}
local ready, reason = vanguard.can_deploy(rule)
if ready then
    vanguard.await(vanguard.deploy(rule))
else
    print("Can't block yet", reason)
end
```

One more perimeter check: an infected neighbour in the **same LAN** does not pass through your router filter. Put a rule on the host, or isolate it. I would rather hear that twice than read another perfectly configured router next to four infected workstations.
