-- Complete power/recovery experiment on the first workstation only.
-- Deliberately leaves it isolated. This is not a full defence policy.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
for _, host in ipairs(map.hosts) do
    if host.type == "WORKSTATION" then
        for _, action in ipairs({vanguard.shield.ISOLATE_HOST, vanguard.act.SHUTDOWN,
            vanguard.act.STARTUP, vanguard.act.RESTORE}) do
            vanguard.await(vanguard.deploy({action=action, target=host.id}))
            print(action, "finished for", host.id, "at", vanguard.ticks())
        end
        break
    end
end
