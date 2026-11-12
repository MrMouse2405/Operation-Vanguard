-- Trusted scheduler facade. Neither raw host functions nor coroutine controls
-- are exported into the agent environment. This code is instruction-metered too.
local host, yield, pack, unpack, rawnext, rawtype, rawerror, rawtostring = ...
local function call(name, ...)
    local args = pack(...)
    while true do
        local result = pack(pcall(host, name, args))
        if not result[1] then
            -- mlua internally wraps callback errors in userdata. Public API
            -- failures must be ordinary, catchable Lua error strings.
            local text = rawtostring(result[2])
            rawerror(text, 0)
        end
        if rawtype(result[2]) == 'string' then rawerror(result[2], 0) end
        if result[2] then return unpack(result, 3, result.n) end
        yield("fuel")
    end
end
local api = {}
for _, name in ipairs({"self", "router", "others", "tx", "rx", "pub", "sub", "unsub",
    "fuel", "ticks", "cost", "deploy", "slots", "tasks", "status", "result",
    "can_deploy", "has_task", "is_finished", "is_ready"}) do
    api[name] = function(...) return call(name, ...) end
end
function api.wait(ticks)
    call("validate_wait", ticks)
    local until_tick = api.ticks() + ticks
    yield("wait", until_tick)
    return true
end
function api.await(id)
    if not api.has_task(id) then rawerror("UNKNOWN_TASK: task is not owned by this router", 2) end
    if not api.is_finished(id) then yield("await", id) end
    return api.result(id)
end
function api.await_any(ids)
    local checked = call("validate_await_any", ids)
    while true do
        for _, id in ipairs(checked) do
            if api.is_finished(id) then return id end
        end
        yield("await_any", checked)
    end
end
-- Stable iteration for supported scalar keys. No pointer-address ordering.
local function keys(t)
    if rawtype(t) ~= "table" then rawerror("INVALID_ARGUMENT: expected table", 3) end
    local list, key = {}, nil
    while true do
        key = rawnext(t, key)
        if key == nil then break end
        local kind = rawtype(key)
        if kind ~= "number" and kind ~= "string" and kind ~= "boolean" then
            rawerror("INVALID_ARGUMENT: deterministic iteration supports scalar keys only", 3)
        end
        list[#list+1] = key
    end
    local rank = {boolean=1, number=2, string=3}
    table.sort(list, function(a,b)
        local ta,tb = rawtype(a),rawtype(b)
        if ta ~= tb then return rank[ta] < rank[tb] end
        if ta == "boolean" then return not a and b end
        return a < b
    end)
    return list
end
local function stable_pairs(t)
    local list, i = keys(t), 0
    return function()
        i = i + 1
        local key = list[i]
        if key ~= nil then return key, t[key] end
    end, t, nil
end
local function stable_next(t, previous)
    local list = keys(t)
    if previous == nil then local key=list[1]; if key ~= nil then return key,t[key] end; return end
    for i,key in ipairs(list) do
        if key == previous then key=list[i+1]; if key ~= nil then return key,t[key] end; return end
    end
    rawerror("INVALID_ARGUMENT: invalid key to next", 2)
end
local module_cache, loading = {}, {}
local function require_module(name)
    if module_cache[name] ~= nil then return module_cache[name] end
    if loading[name] then rawerror("INVALID_ARGUMENT: circular require", 2) end
    local chunk = call("module", name)
    loading[name] = true
    local value = chunk()
    loading[name] = nil
    if value == nil then value = true end
    module_cache[name] = value
    return value
end
return api, stable_pairs, stable_next, require_module, call
