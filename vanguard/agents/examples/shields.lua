-- Complete policy experiment: configure one discovered local workstation.
-- This is a shield demonstration, not a winning defence.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
local workstation, server
for _, host in ipairs(map.hosts) do
    if host.type == "WORKSTATION" and not workstation then workstation = host.id end
    if host.type == "SERVER" and not server then server = host.id end
end
if workstation and server then
    local function apply(action, peer)
        vanguard.await(vanguard.deploy({action=action, target=workstation, peer=peer}))
    end
    apply(vanguard.shield.SET_DEFAULT_DENY_ON_HOST)
    apply(vanguard.shield.ALLOW_PEER_ON_HOST, server)
    print("Workstation may talk only to the local server", workstation, server)
    apply(vanguard.shield.ISOLATE_HOST) -- saved allowlist survives
    apply(vanguard.shield.RELEASE_HOST) -- allowlist applies again
end
