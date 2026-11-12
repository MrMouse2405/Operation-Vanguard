-- Complete, deliberately serial teaching agent. Slow on large LANs.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
local function act(action, host)
    local desc = {action=action, target=host}
    if not vanguard.can_deploy(desc) then return nil end
    local id = vanguard.deploy(desc)
    vanguard.await_any({id})
    if vanguard.is_ready(id) then return vanguard.result(id).data end
end
while true do
    for _, host in ipairs(map.hosts) do
        local scan = act(vanguard.sense.HOST_SCAN, host.id)
        if scan and scan.assessment ~= "CLEAN" then
            if act(vanguard.shield.ISOLATE_HOST, host.id) then
                if act(vanguard.act.RESTORE, host.id) then
                    -- Teaching policy: release immediately. Re-infection is possible.
                    act(vanguard.shield.RELEASE_HOST, host.id)
                end
            end
        end
    end
    vanguard.wait(1)
end
