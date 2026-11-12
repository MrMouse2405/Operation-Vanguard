This is the index I keep open while writing an agent: **23 Lua functions**, grouped by job. The other officers have opinions about how you use them. The linked signatures say exactly what they accept and return.

| I want to… | Open this email | Functions |
| --- | --- | --- |
| Find myself and teammates | [Identity](/docs/api/identity/) | self, router, others |
| Start work and read results | [Tasks](/docs/api/tasks/) | deploy, slots, tasks, status, result, await, await_any |
| Check whether work is possible or done | [Readiness](/docs/api/readiness/) | can_deploy, has_task, is_finished, is_ready |
| Talk to teammates | [Communication](/docs/api/communication/) | tx, rx, pub, sub, unsub |
| Read the clock or check costs | [Time and fuel](/docs/api/time/) | fuel, ticks, wait, cost |

Most invalid calls raise a Lua error starting with a stable [reason label](/docs/errors/). Readiness checks return booleans or reasons for expected problems. Check the individual signature; don't guess the error convention from a different call.

Every call uses fuel. If there isn’t enough, the call may pause *before* doing anything and continue on the next tick. Returned tables are copies of a moment in time, not live views.

## The action goes in the descriptor

To scan, isolate, or restore, pass an action constant to `deploy`:

```lua
local id = vanguard.deploy({action=vanguard.sense.SCAN})
local result = vanguard.await(id)
print(result.data)
```

Keep the action sheets alongside this index: [Scanner’s findings](/docs/actions/scans/), [Firewall’s rules](/docs/actions/shields/), and [Restore’s work orders](/docs/actions/recovery/). The same `deploy` / `await` pattern applies to all three.
