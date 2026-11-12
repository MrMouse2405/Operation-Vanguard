“Can I start it?” “Has it finished?” “Did it succeed?” Three questions. I check them separately. Most trouble begins when somebody treats the first yes as an answer to all three.

These checks still use fuel. Unexpected VM or resource failures remain possible. None reserves a slot.

## vanguard.can_deploy

`vanguard.can_deploy(descriptor) → true` or `false, reason_label`

Checks fields, discovery, ownership, target type, power, locks, and free slots. A malformed request returns `false, "INVALID_ARGUMENT"` instead of raising. Other expected problems return their [reason label](/docs/errors/) too—not a big error object.

Nothing starts or changes. If your code yields before `deploy`, the world may have changed by admission. My earlier yes is not a reservation. Handle the failure there too.

## vanguard.has_task

`vanguard.has_task(task_id) → boolean`

True if your router knows that task ID, including finished tasks. False for invalid, unknown, or another router’s IDs. Handy before `status`, `result`, or `await` when you aren’t sure about an ID. It doesn’t reveal teammates’ jobs.

## vanguard.is_finished

`vanguard.is_finished(task_id) → boolean`

True for `COMPLETED` **or `FAILED`** local tasks. False for running, invalid, unknown, or foreign IDs. A failed restore is finished. I would still like to know whether it worked before releasing the host.

## vanguard.is_ready

`vanguard.is_ready(task_id) → boolean`

True only for a successfully `COMPLETED` local task. You can read its result without a task-failure error. False for running, failed, invalid, unknown, or foreign IDs.

```lua
local ready, reason = vanguard.can_deploy({action=vanguard.sense.SCAN})
if not ready then print("Try later", reason) end

local id = vanguard.deploy({action=vanguard.sense.SCAN})
print(vanguard.has_task(id))
print(vanguard.is_finished(id))
vanguard.await_any({id})
if vanguard.is_ready(id) then
    print(vanguard.result(id))
else
    print("Didn't work", vanguard.status(id).failure)
end
```
