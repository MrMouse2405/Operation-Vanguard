-- Organizer feasibility control. Same code on every router; no topology/seed IDs.
-- Evidence-led source containment, followed by local recovery and a rolling patrol.
-- No dashboard truth, file access, known infected IDs or special engine privileges.
local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
local members, queue, queued, checked, recovering, jobs = {}, {}, {}, {}, {}, {}
for _, host in ipairs(map.hosts) do members[host.id] = true end
local cursor, traffic_at = 1, -100
local function submit(action, target, stage)
    local desc = {action=action, target=target}
    if not vanguard.can_deploy(desc) then return false end
    local id = vanguard.deploy(desc)
    jobs[id] = {host=target, stage=stage}
    return true
end
local function enqueue(host)
    if members[host] and not queued[host] then
        queue[#queue+1] = host
        queued[host] = true
    end
end
while true do
    for id, work in pairs(jobs) do
        if vanguard.is_finished(id) then
            jobs[id] = nil
            if vanguard.is_ready(id) then
                local data = vanguard.result(id).data
                if work.stage == 'traffic' then
                    traffic_at = vanguard.ticks()
                    for _, pair in ipairs(data.pairs) do enqueue(pair.a); enqueue(pair.b) end
                elseif work.stage == 'scan' then
                    checked[work.host] = vanguard.ticks()
                    if data.assessment == 'COMPROMISED' then recovering[work.host] = 'isolate' end
                elseif work.stage == 'isolate' then recovering[work.host] = 'restore'
                elseif work.stage == 'restore' then recovering[work.host] = 'release'
                elseif work.stage == 'release' then recovering[work.host] = nil end
            elseif work.host then
                if work.stage ~= 'scan' then recovering[work.host] = 'isolate' end
            else traffic_at = -100 end
        end
    end
    local busy, scanning = {}, false
    for _, work in pairs(jobs) do
        if work.host then busy[work.host] = true else scanning = true end
    end
    for host, stage in pairs(recovering) do
        if not busy[host] and vanguard.slots().available > 0 then
            local action = stage == 'isolate' and vanguard.shield.ISOLATE_HOST
                or stage == 'restore' and vanguard.act.RESTORE or vanguard.shield.RELEASE_HOST
            if submit(action, host, stage) then busy[host] = true end
        end
    end
    -- History is a lead, not proof of infection; confirm health on local endpoints.
    for index=#queue,1,-1 do
        local host = queue[index]
        if not busy[host] and not recovering[host] then
            if checked[host] and vanguard.ticks()-checked[host] < 12 then
                table.remove(queue,index); queued[host] = nil
            elseif vanguard.slots().available > 0 and submit(vanguard.sense.HOST_SCAN,host,'scan') then
                busy[host] = true; table.remove(queue,index); queued[host] = nil
            end
        end
    end
    if not scanning and vanguard.ticks()-traffic_at >= 4 and vanguard.slots().available > 0 then
        submit(vanguard.sense.TRAFFIC_SCAN,nil,'traffic')
    end
    for _=1,#map.hosts do
        if vanguard.slots().available == 0 then break end
        local host = map.hosts[cursor].id
        cursor = cursor % #map.hosts + 1
        if not busy[host] and not recovering[host] then
            if submit(vanguard.sense.HOST_SCAN,host,'scan') then busy[host] = true end
        end
    end
    vanguard.wait(1)
end
