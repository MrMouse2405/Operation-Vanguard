-- Complete agent. Finds local machines and prints six kinds of evidence.
local function run(desc)
    if not vanguard.can_deploy(desc) then return nil end
    local id = vanguard.deploy(desc)
    vanguard.await_any({id})
    if vanguard.is_ready(id) then return vanguard.result(id).data end
    print("scan failed", vanguard.status(id).failure)
end
local map = run({action=vanguard.sense.SCAN})
print("local map", map)
print("router rules", run({action=vanguard.sense.SHIELD_STATUS}))
print("past transit traffic", run({action=vanguard.sense.TRAFFIC_SCAN}))
for _, host in ipairs(map.hosts) do
    for _, action in ipairs({vanguard.sense.HOST_SCAN, vanguard.sense.CONNECTION_SCAN,
        vanguard.sense.FULL_HOST_SCAN, vanguard.sense.SHIELD_STATUS}) do
        print(host.id, action, run({action=action, target=host.id}))
    end
end
