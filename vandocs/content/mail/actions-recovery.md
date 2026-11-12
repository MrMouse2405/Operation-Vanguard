I can return a clean machine. I cannot make the network you return it to safe. Please arrange the isolation before the restore, and check the neighbourhood before releasing it.

Pulling the plug stops traffic. The infection is still there when you put the plug back.

Use `{action=vanguard.act.ACTION, target=host_id}`. The host must be discovered and in your LAN. Don’t supply `peer`; routers aren’t valid targets. Each job uses a slot and that host’s change lock. Success returns `data={changed=boolean}`.

## SHUTDOWN — 1 tick

At completion the host becomes `OFFLINE`, and its connections close. Services and infection spread stop. **Infection and all shield settings stay.** An already-offline host succeeds with `changed=false`.

## STARTUP — 3 ticks

An offline host becomes `STARTING` at the first turn boundary after admission, then `ONLINE` at completion. It can’t communicate or spread infection while starting.

Already online? Success with `changed=false`. Infection, rules, and isolation stay. If startup later fails, the host is left offline.

## RESTORE — 20 ticks

Works on online or offline hosts. At the first turn boundary, the host becomes `RESTORING`: connections close, services stop, and it can’t spread infection.

At completion, infection is removed and it returns `ONLINE`. **Rules and isolation stay in place.** An unprotected host can get infected again. A failed restore leaves it offline and doesn’t guarantee a cure.

## This is the order I use

```lua
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
for _, host in ipairs(map.hosts) do
    local protect = {action=vanguard.shield.ISOLATE_HOST, target=host.id}
    if vanguard.can_deploy(protect) then
        vanguard.await(vanguard.deploy(protect))
        local restore = {action=vanguard.act.RESTORE, target=host.id}
        if vanguard.can_deploy(restore) then
            vanguard.await(vanguard.deploy(restore))
        end
    end
end
```

I’ve left the hosts isolated in this example. It shows the recovery sequence, not a complete defence: running it across every machine also takes every machine out of service. Your agent needs a decision about `RELEASE_HOST` after the repair. Don’t forget that last job just because the scan looks better.
