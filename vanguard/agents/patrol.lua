-- Shared teaching implementation. Each host has at most one outstanding job.
local M = {}
function M.run(mode)
    local map = vanguard.await(vanguard.deploy({action=vanguard.sense.SCAN})).data
    local hosts, local_host = map.hosts, {}
    for _, host in ipairs(hosts) do local_host[host.id] = true end
    if mode == "advanced" then
        table.sort(hosts, function(a, b)
            if a.type ~= b.type then return a.type == "SERVER" end
            return a.id < b.id
        end)
    end
    local warnings, warned, recovering, blocked = {}, {}, {}, {}
    local transit_at = -16
    if mode ~= "solo" then
        vanguard.sub("confirmed-threats", function(sender, topic, message)
            -- A message is advice, not a new discovery or a remote-control API.
            if type(message) == "table" and type(message.host) == "number" then
                warnings[message.host] = message
            end
        end)
        vanguard.wait(1) -- make the subscription visible to teammates
    end
    local jobs, cursor = {}, 1
    local function submit(action, target, stage)
        local desc = {action=action, target=target}
        if not vanguard.can_deploy(desc) then return false end
        local ok, id = pcall(vanguard.deploy, desc)
        if not ok then return false end
        jobs[id] = {host=target, stage=stage}
        return true
    end
    while true do
        -- Advance every completed state machine before admitting more scans.
        for id, work in pairs(jobs) do
            if vanguard.is_finished(id) then
                jobs[id] = nil
                if vanguard.is_ready(id) then
                    if work.stage == "scan" then
                        local result = vanguard.result(id).data
                        if result.assessment ~= "CLEAN" then
                            recovering[work.host] = "isolate"
                            if mode ~= "solo" and result.assessment == "COMPROMISED" and not warned[work.host] then
                                warned[work.host] = true
                                vanguard.pub("confirmed-threats", {host=work.host, observed_at=result.observed_at})
                            end
                        end
                    elseif work.stage == "isolate" then recovering[work.host] = "restore"
                    elseif work.stage == "restore" then recovering[work.host] = "release"
                    elseif work.stage == "release" then
                        recovering[work.host] = nil
                        if mode ~= "solo" and warned[work.host] then
                            warned[work.host] = nil
                            vanguard.pub("confirmed-threats", {host=work.host, clear=true, observed_at=vanguard.ticks()})
                        end
                    elseif work.stage == "traffic" then
                        transit_at = vanguard.ticks()
                    end
                else
                    -- A failed scan can be retried on the next circuit.
                    -- Recovery failures return to isolation rather than claiming a cure.
                    if work.host and work.stage ~= "scan" then recovering[work.host] = "isolate" end
                    print("job failed", work.host, vanguard.status(id).failure)
                end
            end
        end
        local busy = {}
        for _, work in pairs(jobs) do if work.host then busy[work.host] = true end end
        if mode ~= "solo" then
            -- Scan before applying reports: messages do not grant discovery.
            local scanning = false
            for _, work in pairs(jobs) do if work.stage == "traffic" then scanning = true end end
            if not scanning and vanguard.ticks() - transit_at >= 16 and vanguard.slots().available > 0 then
                submit(vanguard.sense.TRAFFIC_SCAN, nil, "traffic")
            end
            for host, report in pairs(warnings) do
                if not local_host[host] and vanguard.slots().available > 0 then
                    local action = report.clear and vanguard.shield.ALLOW_PEER or vanguard.shield.BLOCK_PEER
                    if (report.clear and blocked[host]) or (not report.clear and not blocked[host]) then
                        local desc = {action=action, peer=host}
                        if vanguard.can_deploy(desc) then
                            local ok, id = pcall(vanguard.deploy, desc)
                            if ok then
                                vanguard.await_any({id})
                                if vanguard.is_ready(id) then blocked[host] = not report.clear end
                            end
                        end
                    end
                    if report.clear and not blocked[host] then warnings[host] = nil end
                end
            end
        end
        for host, stage in pairs(recovering) do
            if not busy[host] and vanguard.slots().available > 0 then
                local action = stage == "isolate" and vanguard.shield.ISOLATE_HOST
                    or stage == "restore" and vanguard.act.RESTORE
                    or vanguard.shield.RELEASE_HOST
                if submit(action, host, stage) then busy[host] = true end
            end
        end
        -- Advanced patrols re-check local hosts mentioned in a warning first.
        if mode == "advanced" then
            for host in pairs(warnings) do
                if local_host[host] and not busy[host] and not recovering[host] then
                    if submit(vanguard.sense.HOST_SCAN, host, "scan") then busy[host] = true; warnings[host] = nil end
                end
            end
        end
        -- One circuit maximum per tick: don't spin when a LAN has no hosts.
        for _ = 1, #hosts do
            if vanguard.slots().available == 0 then break end
            local host = hosts[cursor].id
            cursor = cursor % #hosts + 1
            if not busy[host] and not recovering[host] then
                if submit(vanguard.sense.HOST_SCAN, host, "scan") then busy[host] = true end
            end
        end
        vanguard.wait(1)
    end
end
return M
