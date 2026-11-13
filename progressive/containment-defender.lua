-- Contain first, diagnose safely, then restore in a bounded pipeline.
-- Normal Lua API only: no fixture IDs, hidden state, or topology-specific plans.
-- Traffic history is a lead, not proof. Intra-LAN-only attacks need the patrol.
local jobs, hosts, members = {}, {}, {}
local restores, traffic_due, traffic_running, cursor = 0, 0, false, 1
local map_ready = false

local function submit(action, host, stage)
    local descriptor = {action=action}
    if host then descriptor.target = host.id end
    if not vanguard.can_deploy(descriptor) then return false end
    local ok, id = pcall(vanguard.deploy, descriptor)
    if not ok then return false end
    jobs[#jobs+1] = {id=id, host=host, stage=stage}
    if host then host.task = id end
    if stage == "restore" then restores = restores + 1 end
    if stage == "traffic" then traffic_running = true end
    return true
end

local function evidence(id, peer)
    local host = members[id]
    if not host or host.peers[peer] then return end
    host.peers[peer] = true
    host.degree = host.degree + 1
    if not host.stage then host.stage = "quarantine" end
end

local function complete(work, data, now)
    local host = work.host
    if work.stage == "map" then
        for _, node in ipairs(data.hosts) do
            local record = {id=node.id, kind=node.type, peers={}, degree=0, checked=-1000, confirmed=false}
            members[node.id] = record
            hosts[#hosts+1] = record
        end
        table.sort(hosts, function(a,b) return a.id < b.id end)
        map_ready = true
    elseif work.stage == "traffic" then
        -- Retain only locally relevant edges, not the whole transit history.
        -- A repeated historical pair does not quarantine a successfully cleared
        -- host forever; the independent health patrol still catches recurrence.
        for _, pair in ipairs(data.pairs) do
            evidence(pair.a, pair.b)
            evidence(pair.b, pair.a)
        end
        traffic_due = now + 4
    elseif work.stage == "quarantine" then
        host.stage = host.confirmed and "restore" or "diagnose"
    elseif work.stage == "diagnose" or work.stage == "patrol" then
        host.checked = now
        host.confirmed = data.assessment == "COMPROMISED"
        if work.stage == "diagnose" then
            host.stage = host.confirmed and "restore" or "release"
        elseif data.assessment ~= "CLEAN" then
            host.stage = "quarantine"
        end
    elseif work.stage == "restore" then
        host.confirmed = false
        host.stage = "release"
    elseif work.stage == "release" then
        host.stage = nil
        host.checked = now
        host.confirmed = false
    end
end

submit(vanguard.sense.SCAN, nil, "map")
-- Overlap discovery and traffic sensing instead of losing the first six ticks.
submit(vanguard.sense.TRAFFIC_SCAN, nil, "traffic")

while true do
    local now = vanguard.ticks()
    for index=#jobs,1,-1 do
        local work = jobs[index]
        if vanguard.is_finished(work.id) then
            table.remove(jobs,index)
            if work.host then work.host.task = nil end
            if work.stage == "restore" then restores = restores - 1 end
            if work.stage == "traffic" then traffic_running = false end
            if vanguard.is_ready(work.id) then
                complete(work,vanguard.result(work.id).data,now)
            else
                -- Failed tasks never establish a cure or a successful release.
                if work.stage == "traffic" then traffic_due = now end
                if work.host and work.stage ~= "patrol" then
                    work.host.stage = work.stage
                end
                print("containment task failed",work.stage,vanguard.status(work.id).failure)
            end
        end
    end

    if map_ready then
        local available = vanguard.slots().available
        -- Restore service promptly. Clean quarantined endpoints and completed
        -- restores must not wait behind another batch of long-running work.
        for _, host in ipairs(hosts) do
            if available == 0 then break end
            if host.stage == "release" and not host.task then
                if submit(vanguard.shield.RELEASE_HOST,host,"release") then available=available-1 end
            end
        end

        local urgent = {}
        for _, host in ipairs(hosts) do
            if host.stage == "quarantine" and not host.task then urgent[#urgent+1]=host end
        end
        -- Degree is only a scheduling hint. Every candidate is contained and
        -- examined; a high-degree clean target is never labelled infected.
        table.sort(urgent,function(a,b)
            if a.confirmed ~= b.confirmed then return a.confirmed == true end
            if a.degree ~= b.degree then return a.degree > b.degree end
            return a.id < b.id
        end)
        for _, host in ipairs(urgent) do
            if available == 0 then break end
            if submit(vanguard.shield.ISOLATE_HOST,host,"quarantine") then available=available-1 end
        end

        -- Diagnose servers first to release unnecessary service outages quickly.
        for pass=1,2 do
            for _, host in ipairs(hosts) do
                if available == 0 then break end
                if host.stage == "diagnose" and not host.task
                    and (host.kind == "SERVER") == (pass == 1) then
                    if submit(vanguard.sense.HOST_SCAN,host,"diagnose") then available=available-1 end
                end
            end
        end

        if available > 0 and not traffic_running and now >= traffic_due then
            if submit(vanguard.sense.TRAFFIC_SCAN,nil,"traffic") then available=available-1 end
        end

        -- At most three 20-tick restores: retain two slots for urgent containment,
        -- evidence and releases. Filling all five slots caused the old patrol's
        -- outbreak to outrun its ability to act on newly discovered threats.
        for _, host in ipairs(hosts) do
            if available == 0 or restores >= 3 then break end
            if host.stage == "restore" and not host.task then
                if submit(vanguard.act.RESTORE,host,"restore") then available=available-1 end
            end
        end

        -- Fair rolling coverage catches silent/local-only sources as well as
        -- reinfection on an already-seen pair. Never scan a host being restored.
        for _=1,#hosts do
            if available == 0 then break end
            local host = hosts[cursor]
            cursor = cursor % #hosts + 1
            if not host.stage and not host.task and now-host.checked >= 12 then
                if submit(vanguard.sense.HOST_SCAN,host,"patrol") then available=available-1 end
            end
        end
    end
    vanguard.wait(1)
end
