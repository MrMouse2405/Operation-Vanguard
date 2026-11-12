“It broke” is not an error report. Send the reason prefix, the task ID if one exists, and the tick. Those let us distinguish a bad request from a perfectly good request to a host that went offline while you waited.

Most invalid calls raise a Lua error string beginning with a reason label. The same names are in `vanguard.error`.

Don't sit waiting for `(nil, error)` returns; ordinary failures raise. `can_deploy` returns `false, reason_label` for expected preflight problems. That distinction belongs in your error handling.

| Prefix | What to check |
| --- | --- |
| INVALID_ARGUMENT | Wrong type, ID, list, topic, payload, or extra descriptor field |
| UNKNOWN_ACTION | That action name isn’t registered |
| UNKNOWN_AGENT | No teammate has that agent ID |
| UNKNOWN_NODE | No node has that ID |
| UNDISCOVERED | Your agent hasn’t learned that node or peer yet |
| OUT_OF_SCOPE | That host belongs to another router |
| INVALID_TARGET_TYPE | You used a router where a host was needed, or another wrong target shape |
| OFFLINE | The action needs a live host, but it’s off |
| TARGET_BUSY | Another change holds the lock, or the host is STARTING/RESTORING |
| TASK_SLOTS_FULL | All five local slots are in use |
| UNKNOWN_TASK | That task ID is invalid, unknown, or belongs to another router |
| TASK_NOT_FINISHED | You asked for a result while the task is still RUNNING |
| PAYLOAD_TOO_LARGE | A payload or native value is bigger than the configured limit |
| LIMIT_EXCEEDED | A fuel, ID, or explicit runtime resource limit was reached |

## Keep the actual error

```lua
local ok, id_or_error = pcall(function()
    return vanguard.deploy({action=vanguard.sense.SCAN})
end)
if not ok then print("Couldn't start scan", id_or_error) end
```

`pcall` returns whether the function succeeded, then its result or error. A rejected admission creates **no job**.

## Accepted at seven. Failed at ten.

A host can go offline before the scan finishes. Read `status(id).failure` and `finished_at` before accusing admission of making up a task. The request and the later result happened at different ticks.

`await_any` returns when a task completes **or fails**. `is_ready` tells you whether it succeeded. `result` and `await` raise the saved failure.

A message-handler error stops that handler and logs sender/message context. Its admitted tasks keep running. Account for them before retrying the whole operation; your “second attempt” may be competing with work from the first.
