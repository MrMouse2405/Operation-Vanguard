I have **five slots per router**. Scans go in those slots. Restores go in those slots. The job your message handler just started goes in those slots. There is no sixth slot under the desk.

A **task** asks the engine to do work. When I give you its ID, I have accepted it. I have not finished it.

## Put the request in this shape

```lua
local descriptor = {
    action = vanguard.sense.HOST_SCAN,
    target = host_id,
}
```

Only `action`, `target`, and `peer` are allowed. `host_id` needs to be a discovered host in your LAN.

Host actions need `target`. Router actions use your own router automatically and **reject any target field**, even your router’s ID. `SHIELD_STATUS` is the exception: no target checks your router; a target checks a local host.

Only peer-blocking and peer-allowing actions accept `peer`. There’s no priority, window, cursor, or cancellation field.

## How the desk fills up

Each router has **five shared task slots**, including jobs started by message handlers. A sixth job raises `TASK_SLOTS_FULL`; it isn’t queued.

Only one mutation can hold a target’s lock at a time. Router shield changes share the router lock; recovery and shield changes on one host share that host’s lock. Conflicts get `TARGET_BUSY`. Scans don’t take those locks, so you can inspect while work is underway.

Tasks go from `RUNNING` to `COMPLETED` or `FAILED`. Finished jobs release their slots and locks at the turn boundary. Their records stay readable for the rest of the run. Finishing your code, or a handler crashing, doesn’t cancel jobs already started.

## Collect your receipt, then your result

```lua
local task = {action=vanguard.sense.HOST_SCAN, target=host_id}
local ready, reason = vanguard.can_deploy(task)
if ready then
    local id = vanguard.deploy(task)
    vanguard.await_any({id}) -- done, but maybe failed
    if vanguard.is_ready(id) then
        print(vanguard.result(id).data)
    else
        print("Task failed", vanguard.status(id).failure)
    end
else
    print("Not started", reason)
end
```

Successful results look like this (`fuel_spent` below is just an example):

```lua
{
    task_id=1, action="HOST_SCAN", target=1001, router_id=1,
    started_at=7, finished_at=10, fuel_spent=100,
    data={observed_at=10, power="ONLINE", assessment="SUSPICIOUS"},
}
```

Router-only results omit `target`. Changes return `data={changed=boolean}`. Doing something already done may return `changed=false`, but still costs time, fuel, and a slot.

## A quote does not hold your place

`can_deploy` and `cost` are checks, not reservations. The engine checks discovery, ownership, and power again at admission and completion. Scans see the world **when they finish**.

Live host scans need `ONLINE`. An offline host gives `OFFLINE`; a starting or restoring host gives `TARGET_BUSY`. `SHIELD_STATUS` works in any power state.

`result` and `await` raise a failed task’s error. `status` gives you the failure as data. Keep the task ID when you [write to Debug](/docs/errors/); “the scan” is not a filing system.
