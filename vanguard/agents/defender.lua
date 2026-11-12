-- Conservative example, not an autonomous engine-side defender. Only local,
-- discovered hosts may be managed. At most one mutation per host is admitted.
require("team")
local topology = require("discovery")
while true do
    for _, host in ipairs(topology.hosts) do
        local scan = {action=vanguard.sense.HOST_SCAN, target=host.id}
        if vanguard.can_deploy(scan) then
            local id = vanguard.deploy(scan)
            vanguard.await_any({id})
            if vanguard.is_ready(id) then
                local result = vanguard.result(id)
                print("assessment", {host=host.id, result=result.data})
                if result.data.assessment ~= vanguard.health.CLEAN then
                    vanguard.pub("threats", {host=host.id, assessment=result.data.assessment})
                    local isolate = {action=vanguard.shield.ISOLATE_HOST, target=host.id}
                    if vanguard.can_deploy(isolate) then
                        vanguard.await(vanguard.deploy(isolate))
                        local restore = {action=vanguard.act.RESTORE, target=host.id}
                        if vanguard.can_deploy(restore) then
                            vanguard.await(vanguard.deploy(restore))
                            -- Keep isolation until the operator/agent chooses to
                            -- release it. Restoration does not reset shield rules.
                        end
                    end
                end
            else
                print("scan failed", vanguard.status(id))
            end
        end
    end
    vanguard.wait(1)
end
