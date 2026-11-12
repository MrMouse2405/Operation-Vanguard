-- Complete observed-session reconstruction. No hidden topology/health reads.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
while true do
    if dbgguard then
        dbgguard.view("connections")
        dbgguard.clear("connections")
    end
    for _, host in ipairs(map.hosts) do
        local desc = {action=vanguard.sense.CONNECTION_SCAN, target=host.id}
        if vanguard.can_deploy(desc) then
            local id = vanguard.deploy(desc)
            vanguard.await_any({id})
            if dbgguard and vanguard.is_ready(id) then
                local data = vanguard.result(id).data
                dbgguard.node("connections", host.id, {kind=host.type, observed_at=data.observed_at})
                for index, connection in ipairs(data.connections) do
                    dbgguard.node("connections", connection.peer, {kind="UNKNOWN", observed_at=data.observed_at})
                    local from, to = host.id, connection.peer
                    if connection.direction == "INBOUND" then from, to = to, from end
                    dbgguard.edge("connections", host.id .. "-" .. index, from, to,
                        {direction=connection.direction, opened_at=connection.opened_at, observed_at=data.observed_at})
                end
            end
        end
    end
    if dbgguard then dbgguard.note("connections", {refresh_finished=vanguard.ticks(), warning="Scans complete at different ticks"}) end
    vanguard.wait(1)
end
