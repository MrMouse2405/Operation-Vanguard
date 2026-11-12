-- Trusted, pure-Lua sequence helpers: their work is metered with the agent.
local table = {}

function table.insert(values, ...)
    local args = {...}
    local count = #values
    local position, value
    local arity = select("#", ...)
    assert(arity == 1 or arity == 2, "table.insert expects two or three arguments")
    if arity == 1 then
        position, value = count + 1, args[1]
    else
        position, value = args[1], args[2]
    end
    assert(type(position) == "number" and position % 1 == 0
        and position >= 1 and position <= count + 1, "position out of bounds")
    for i = count, position, -1 do values[i + 1] = values[i] end
    values[position] = value
end

function table.remove(values, position)
    local count = #values
    position = position or count
    if count == 0 and position == 0 then return nil end
    assert(type(position) == "number" and position % 1 == 0
        and position >= 1 and position <= count, "position out of bounds")
    local value = values[position]
    for i = position, count - 1 do values[i] = values[i + 1] end
    values[count] = nil
    return value
end

function table.concat(values, separator, first, last)
    separator, first, last = separator or "", first or 1, last or #values
    local result = ""
    for i = first, last do
        local value = values[i]
        assert(type(value) == "string" or type(value) == "number",
            "table.concat expects strings or numbers")
        if i > first then result = result .. separator end
        result = result .. value
    end
    return result
end

-- Powersort scheduling and minrun computation adapted from CPython 3.14.0:
-- https://github.com/python/cpython/blob/v3.14.0/Objects/listobject.c
-- See licenses/CPython.txt for attribution, license, and adaptation notes.
local function minrun_length(count)
    local remainder = 0
    while count >= 64 do
        remainder = remainder | (count & 1)
        count = count >> 1
    end
    return count + remainder
end

-- Depth of the first differing bit in the normalized adjacent-run midpoints.
-- CPython's powerloop uses zero-based offsets, unlike the table itself.
local function run_power(first, left_count, right_count, count)
    local a = 2 * (first - 1) + left_count
    local b = a + left_count + right_count
    local power = 0
    while true do
        power = power + 1
        if a >= count then
            a, b = a - count, b - count
        elseif b >= count then
            return power
        end
        a, b = a * 2, b * 2
    end
end

local function natural_run(values, first, last, less)
    if first == last then return 1 end
    local stop = first + 1
    if less(values[stop], values[first]) then
        -- Strictly descending: reversing equal elements would break stability.
        while stop < last and less(values[stop + 1], values[stop]) do
            stop = stop + 1
        end
        local left, right = first, stop
        while left < right do
            values[left], values[right] = values[right], values[left]
            left, right = left + 1, right - 1
        end
    else
        while stop < last and not less(values[stop + 1], values[stop]) do
            stop = stop + 1
        end
    end
    return stop - first + 1
end

local function extend_run(values, first, sorted_end, last, less)
    for index = sorted_end + 1, last do
        local pivot = values[index]
        local left, right = first, index
        while left < right do
            local middle = left + (right - left) // 2
            if less(pivot, values[middle]) then
                right = middle
            else
                -- Insert after existing equals to keep the sort stable.
                left = middle + 1
            end
        end
        for move = index, left + 1, -1 do values[move] = values[move - 1] end
        values[left] = pivot
    end
end

local function merge_runs(values, left, right, less)
    local first, middle = left.first, right.first
    local last = middle + right.count - 1
    if not less(values[middle], values[middle - 1]) then return end

    -- Buffer only the shorter run. Forward merging takes left-side equals
    -- first; backward merging takes right-side equals first.
    local buffer = {}
    if left.count <= right.count then
        for i = 1, left.count do buffer[i] = values[first + i - 1] end
        local a, b, dest = 1, middle, first
        while a <= left.count and b <= last do
            if less(values[b], buffer[a]) then
                values[dest], b = values[b], b + 1
            else
                values[dest], a = buffer[a], a + 1
            end
            dest = dest + 1
        end
        while a <= left.count do
            values[dest], a, dest = buffer[a], a + 1, dest + 1
        end
    else
        for i = 1, right.count do buffer[i] = values[middle + i - 1] end
        local a, b, dest = middle - 1, right.count, last
        while a >= first and b >= 1 do
            if less(buffer[b], values[a]) then
                values[dest], a = values[a], a - 1
            else
                values[dest], b = buffer[b], b - 1
            end
            dest = dest - 1
        end
        while b >= 1 do
            values[dest], b, dest = buffer[b], b - 1, dest - 1
        end
    end
end

function table.sort(values, compare)
    assert(type(values) == "table", "table.sort expects a table")
    assert(compare == nil or type(compare) == "function",
        "table.sort comparator must be a function")
    local less = compare or function(a, b) return a < b end
    local count = #values
    if count < 2 then return end
    local minrun = minrun_length(count)
    local runs = {}

    local function merge_at(index)
        local left, right = runs[index], runs[index + 1]
        merge_runs(values, left, right, less)
        left.count = left.count + right.count
        for i = index + 1, #runs - 1 do runs[i] = runs[i + 1] end
        runs[#runs] = nil
    end

    local first = 1
    while first <= count do
        local length = natural_run(values, first, count, less)
        local target = minrun
        if target > count - first + 1 then target = count - first + 1 end
        if length < target then
            extend_run(values, first, first + length - 1, first + target - 1, less)
            length = target
        end

        local top = #runs
        if top > 0 then
            local previous = runs[top]
            local power = run_power(previous.first, previous.count, length, count)
            -- CPython's found_new_run: pending boundary powers increase from
            -- bottom to top. Merge deeper boundaries before pushing this run.
            while #runs > 1 and runs[#runs - 1].power > power do
                merge_at(#runs - 1)
            end
            runs[#runs].power = power
        end
        runs[#runs + 1] = {first = first, count = length}
        first = first + length
    end

    -- CPython's merge_force_collapse chooses the smaller outside neighbor.
    while #runs > 1 do
        local index = #runs - 1
        if index > 1 and runs[index - 1].count < runs[index + 1].count then
            index = index - 1
        end
        merge_at(index)
    end
end

return table
