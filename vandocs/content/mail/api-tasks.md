These are the calls for my desk: submit, inspect, collect. **Your router’s jobs only.** A teammate's task number does not transfer the job to you. [The earlier memo](/docs/tasks/) covers slots and locks.

## vanguard.deploy

`vanguard.deploy(descriptor) → positive integer task_id`

Starts a `RUNNING` task in one of your five slots and returns its ID. It doesn’t wait for completion. Bad requests raise their reason label; no free slot raises `TASK_SLOTS_FULL`; a conflicting change raises `TARGET_BUSY`.

A rejected task creates no job. There’s no queue or cancellation. If fuel is short, admission can pause before making changes, check again later, and execute once. An accepted task then progresses on its own each tick.

## vanguard.slots

`vanguard.slots() → {capacity=5, running, available}`

A snapshot of your slots, including jobs just started by this VM. Finished jobs don’t count as running. Checking this reserves nothing and uses no slot.

## vanguard.tasks

`vanguard.tasks() → integer[]`

Sorted IDs of your currently `RUNNING` tasks. Finished jobs aren’t listed, but you can still read their status and results.

## vanguard.status

`vanguard.status(task_id) → table`

Returns `{id, action, router_id, target?, state, started_at, finished_at?, fuel_spent, failure?}`. The `?` means that field may be absent. Running tasks don’t have `finished_at`.

Unknown or another router’s IDs raise `UNKNOWN_TASK`. Reading a failed task’s status **doesn’t raise its failure**—you get it in `failure`, with its real finish tick.

## vanguard.result

`vanguard.result(task_id) → result_table`

Doesn’t wait. A completed task returns a copy of its result. A running one raises `TASK_NOT_FINISHED`; a failed one raises its saved error; an unknown or foreign one raises `UNKNOWN_TASK`.

Keep the result if you need it. Read it again if you lose your copy. I do not cross completed work off the register when somebody looks at it.

## vanguard.await

`vanguard.await(task_id) → result_table`

Pauses **just the calling coroutine** until the task finishes. Other handlers and tasks keep going. Already finished? It returns or raises immediately.

It returns the same successful shape as `result` and raises the same errors. Waiting uses no extra slot and doesn’t turn task duration into extra fuel.

## vanguard.await_any

`vanguard.await_any(task_ids) → integer task_id`

Waits until any listed task is `COMPLETED` **or `FAILED`**, then returns its ID. It doesn’t raise that task’s failure or consume its result. If several are done, you get the lowest ID.

Use a nonempty dense list. All IDs are checked before waiting. Bad lists raise `INVALID_ARGUMENT`; unknown or foreign tasks raise `UNKNOWN_TASK`; duplicates are ignored. **Remove the returned ID before waiting again.** Otherwise you keep presenting the same finished receipt, and I keep giving you the same answer.

```lua
local a = vanguard.deploy({action=vanguard.sense.SCAN})
local b = vanguard.deploy({action=vanguard.sense.SHIELD_STATUS})
print(vanguard.slots())
local first = vanguard.await_any({b,a,a})
if vanguard.is_ready(first) then print(vanguard.result(first)) end

for _, id in ipairs(vanguard.tasks()) do
    local status = vanguard.status(id)
    print(id, status.action, status.state, status.started_at)
end
print("Free slots", vanguard.slots().available)
```
