-- Complete observation-only agent. Works with and without --dbg.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
if dbgguard then
    dbgguard.view("discovery")
    dbgguard.view("assessments")
    for _, view in ipairs({"discovery", "assessments"}) do
        dbgguard.node(view, vanguard.router(), {kind="ROUTER", observed_at=map.observed_at})
        for _, host in ipairs(map.hosts) do
            dbgguard.node(view, host.id, {kind=host.type, health="UNKNOWN"})
            dbgguard.edge(view, "lan-" .. host.id, vanguard.router(), host.id, {kind="LAN membership"})
        end
    end
end
while true do
    for _, host in ipairs(map.hosts) do
        local desc = {action=vanguard.sense.HOST_SCAN, target=host.id}
        if vanguard.can_deploy(desc) then
            local id = vanguard.deploy(desc)
            vanguard.await_any({id})
            if dbgguard and vanguard.is_ready(id) then
                local data = vanguard.result(id).data
                dbgguard.node("assessments", host.id, {kind=host.type, assessment=data.assessment, observed_at=data.observed_at})
                dbgguard.note("assessments", {last_host=host.id, tick=data.observed_at})
            end
        end
    end
    vanguard.wait(1)
end
