Attached: the names and durations. Keep the spelling. Keep the underscores. There is no `RESTORE_BUT_FASTER`.

`vanguard.VERSION`: `"1.3"`. `vanguard.MAX_TASKS`: **5**. Enum values are strings matching their keys. Use the named constants.

| Namespace | Values |
| --- | --- |
| `vanguard.node_type` | ROUTER, WORKSTATION, SERVER |
| `vanguard.power` | ONLINE, OFFLINE, STARTING, RESTORING |
| `vanguard.health` | CLEAN, SUSPICIOUS, COMPROMISED (scan assessments) |
| `vanguard.task_state` | RUNNING, COMPLETED, FAILED |
| `vanguard.policy` | DEFAULT_ALLOW, DEFAULT_DENY |
| `vanguard.sense` | SCAN, TRAFFIC_SCAN, HOST_SCAN, CONNECTION_SCAN, FULL_HOST_SCAN, SHIELD_STATUS |
| `vanguard.act` | SHUTDOWN, STARTUP, RESTORE |
| `vanguard.shield` | SET_DEFAULT_ALLOW, SET_DEFAULT_DENY, BLOCK_PEER, ALLOW_PEER, SET_DEFAULT_ALLOW_ON_HOST, SET_DEFAULT_DENY_ON_HOST, BLOCK_PEER_ON_HOST, ALLOW_PEER_ON_HOST, ISOLATE_HOST, RELEASE_HOST |

## vanguard.tick_cost

Unit: **ticks**. Not fuel. Not seconds. `cost(descriptor)` also returns the duration after validating the request. Clock has a longer explanation; these are the numbers.

| Action | Ticks |
| --- | --- |
| SCAN | 2 |
| TRAFFIC_SCAN | 4 |
| HOST_SCAN | 3 |
| CONNECTION_SCAN | 3 |
| FULL_HOST_SCAN | 8 |
| SHIELD_STATUS | 1 |
| SHUTDOWN | 1 |
| STARTUP | 3 |
| RESTORE | 20 |
| SET_DEFAULT_ALLOW | 1 |
| SET_DEFAULT_DENY | 1 |
| BLOCK_PEER | 1 |
| ALLOW_PEER | 1 |
| SET_DEFAULT_ALLOW_ON_HOST | 1 |
| SET_DEFAULT_DENY_ON_HOST | 1 |
| BLOCK_PEER_ON_HOST | 1 |
| ALLOW_PEER_ON_HOST | 1 |
| ISOLATE_HOST | 1 |
| RELEASE_HOST | 1 |

```lua
print(vanguard.tick_cost.RESTORE) -- 20
```
