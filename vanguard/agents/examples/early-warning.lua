-- A confirmed local infection is reported to other routers.
-- A receiver learns remote IDs through TRAFFIC_SCAN before trying BLOCK_PEER.
-- Receiving a message alone does NOT discover a node.
local reports, blocked = {}, {}
vanguard.sub("warnings", function(sender, topic, message)
    if type(message) == "table" and type(message.host) == "number" then
        reports[message.host] = message
    end
end)
vanguard.wait(1)
local function job(desc)
    if not vanguard.can_deploy(desc) then return nil end
    local id = vanguard.deploy(desc)
    vanguard.await_any({id})
    if vanguard.is_ready(id) then return vanguard.result(id).data end
end
local map = job({action=vanguard.sense.SCAN})
while true do
    -- History discovers the original host endpoints that traversed this router.
    job({action=vanguard.sense.TRAFFIC_SCAN})
    for host, report in pairs(reports) do
        if not blocked[host] then
            -- Router actions omit target. This blocks transit, not same-LAN traffic.
            if job({action=vanguard.shield.BLOCK_PEER, peer=host}) then
                blocked[host] = true
                print("blocked reported source", host, report.observed_at)
            end
        end
    end
    for _, host in ipairs(map.hosts) do
        local scan = job({action=vanguard.sense.HOST_SCAN, target=host.id})
        if scan and scan.assessment == "COMPROMISED" then
            local connections = job({action=vanguard.sense.CONNECTION_SCAN, target=host.id})
            print("confirmed source and its peers", host.id, connections)
            vanguard.pub("warnings", {host=host.id, observed_at=scan.observed_at})
            job({action=vanguard.shield.ISOLATE_HOST, target=host.id})
        end
    end
    vanguard.wait(1)
end
