-- All routers run this file. Lowest agent ID becomes the coordinator.
-- The leader cannot mutate another router's hosts; it sends a request to the owner.
local me, leader = vanguard.self(), vanguard.self()
for agent in pairs(vanguard.others()) do leader = math.min(leader, agent) end
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
local local_host, orders = {}, {}
for _, host in ipairs(map.hosts) do local_host[host.id] = true end
vanguard.rx(function(sender, message)
    if type(message) ~= "table" then return end
    if me == leader and message.kind == "report" then
        if sender == me then orders[message.host] = true
        else vanguard.tx(sender, {kind="isolate", host=message.host}) end
    elseif sender == leader and message.kind == "isolate" and local_host[message.host] then
        orders[message.host] = true -- only the main patrol mutates hosts
    end
end)
while true do
    for _, host in ipairs(map.hosts) do
        local desc = {action=vanguard.sense.HOST_SCAN, target=host.id}
        if vanguard.can_deploy(desc) then
            local id = vanguard.deploy(desc)
            vanguard.await_any({id})
            if vanguard.is_ready(id) and vanguard.result(id).data.assessment ~= "CLEAN" then
                if me == leader then orders[host.id] = true
                else vanguard.tx(leader, {kind="report", host=host.id}) end
            end
        end
    end
    for host in pairs(orders) do
        local desc = {action=vanguard.shield.ISOLATE_HOST, target=host}
        if vanguard.can_deploy(desc) then
            vanguard.await(vanguard.deploy(desc))
            orders[host] = nil
        end
    end
    vanguard.wait(1)
end
