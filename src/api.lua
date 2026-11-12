---@meta
-- OPERATION VANGUARD — LUA API SPECIFICATION
-- Specification only. Empty functions describe Rust-provided bindings.
-- Do not execute this file as a replacement for the simulator bindings.

local vanguard = {}

vanguard.VERSION = "1.3"
vanguard.MAX_TASKS = 5

--[[
GENERAL RULES

1. Exactly one independent Lua VM runs on each router for the entire run.
   Five routers mean five VMs with private Lua memory, globals, and coroutines.
   Every deployed task executes on its originating router; a host target names
   what that local task operates on, not a new execution location or VM.
   Each Vanguard script is deployed on one router for the entire run.
   Router actions implicitly address that router. No action can inspect,
   configure, shut down, restore, or isolate another router.

2. Agent IDs identify Vanguards. Node IDs identify routers, workstations,
   and servers. These are separate namespaces; all IDs are positive integers.

3. Host actions require an explicit target: a workstation or server belonging
   to the deployment router's LAN. Remote hosts return OUT_OF_SCOPE even when
   their IDs are known. Routers cannot be infected and are not host targets.
   Local host membership is fixed for the run.

4. SCAN discovers local hosts and neighbouring routers. TRAFFIC_SCAN and
   CONNECTION_SCAN also reveal endpoint IDs. Learning a remote ID does not
   grant permission to act there. Known remote peers may be blocked locally.
   Initial knowledge includes the deployment router and teammate agent IDs.

5. Invalid API calls raise ordinary Lua errors (string messages), usable with
   pcall/xpcall. Functions never return nil,error pairs. Errors include a stable
   reason prefix such as "TASK_SLOTS_FULL: all five slots are occupied".
   A rejected submission creates no task. A task admitted successfully can fail
   later; status() reports that failure as data, not as a thrown background error.
   result()/await() raise if the task failed. Preflight functions and status()
   let scripts avoid expected errors. Checks do not reserve resources and cannot
   guarantee a later task succeeds after the world changes.

6. Every task consumes one of the router's FIVE shared slots, regardless of
   action type. deploy() submits asynchronously. There is no pending queue.
   Submission validates and allocates a slot atomically. A sixth submission
   returns TASK_SLOTS_FULL. Completion or failure at a barrier releases a slot.
   Terminal results do not occupy slots. Cancellation is not part of this API.

7. Task states: RUNNING -> COMPLETED or FAILED. Tasks cannot be restarted.
   IDs are unique for the run. Task inspection is restricted to tasks submitted
   on this router. Results remain readable until the simulation ends.

8. SYNCHRONIZED TICKS
   All router VMs advance through tick T before ANY starts tick T+1.
   Each VM receives the same configured per-tick fuel allowance, shared by its
   main coroutine, handlers, and native calls. Unused allowance is not banked.
   Exhausting it suspends the VM until the next tick. A busy loop cannot prevent
   synchronization. A tick is simulation time, not a wall-clock interval.
   Physical CPU threads and their execution order must not change the outcome.

   Tick phases:
     A. Each VM starts with committed world snapshot W[T] and delivered inbox T.
     B. Every VM runs one tick slice, submits local router tasks, and buffers
        outgoing messages and world-change intents. No VM mutates shared W[T].
     C. Barrier: all router VMs finish their tick allowance or become idle/waiting.
     D. Resolve due task mutations in deterministic order; advance network/enemy
        activity for the interval; complete due scans against the resulting
        snapshot W[T+1]. Mutations therefore precede interval activity, and scans
        include that activity. Publish world state and task outcomes together.
     E. Synchronize outgoing messages, topic membership, and teammate fuel totals.
        Release terminal task slots and wake eligible coroutines for T+1.

   A message sent in T can first execute a recipient handler in T+1. A response
   sent by that handler can first arrive in T+2. No cross-VM same-tick delivery.
   Idle/waiting VMs participate automatically; returning from the main script
   does not stop that router's tasks or registered handlers.

9. FUEL AND ROUTER TASK PROGRESS
   ticks() returns T, constant during a tick slice. fuel() is cumulative agent
   fuel. Self fuel includes this slice; teammate fuel is the last synchronized
   value. The scenario publishes the per-tick allowance and native/task costs.
   More Lua computation leaves less fuel to issue actions during the same tick.

   Admitted tasks occupy this router's slots and have positive integer duration
   D. A task admitted in T becomes terminal no earlier than boundary T+D.
   Router tasks progress once per barrier, even if their submitting coroutine
   is waiting. Up to five task durations overlap. Tasks are not new Lua VMs.

   Task fuel is charged once on admission, independently of its elapsed duration;
   waiting does not charge that duration again as equivalent computation.
   Native calls/admission yield BEFORE side effects if their atomic charge will
   not fit the remaining allowance, then resume with a later tick's allowance.
   Published maximum atomic charges must fit one complete tick allowance.
   Slot reservations, locks, and requests survive suspension. Failed submissions
   incur call costs only; admitted tasks do not refund fuel on later failure.

10. VISIBILITY AND DETERMINISM
    Local bookkeeping is visible immediately inside a VM: task admissions, slot
    reservations, locks, handler registrations, and outgoing messages.
    World reads use W[T]; task world effects/completions are visible only after
    the barrier. Scans observe the completion-boundary snapshot. Preconditions
    are checked again then. No scan freezes or reserves world state.
    started_at is admission tick T; finished_at is completion-boundary tick.
    Global task IDs are allocated deterministically from router ID and local
    submission sequence, never CPU timing. Simultaneous mutations use router ID,
    then local submission sequence. Await suspends a coroutine, not world time.

11. One state-changing task may hold a given target at a time. Submitting another
    state-changing task on that same target returns TARGET_BUSY. Router shield
    changes share the router target; host recovery and host shield changes share
    the host target. Read-only scans do not acquire this exclusive lock.
    Locks are released when tasks complete or fail.

12. Defender management and team messaging use a separate control channel.
    Network filtering, host shutdown, and host isolation do not disable it.
    Live scans of OFFLINE hosts fail OFFLINE; scans of STARTING or RESTORING
    hosts fail TARGET_BUSY. SHIELD_STATUS remains available in every power state.

13. Host membership and discovery describe topology, not current connectivity.
    Shutdown or filtering does not remove a node from SCAN results.
]]

vanguard.node_type = {
    ROUTER = "ROUTER",
    WORKSTATION = "WORKSTATION",
    SERVER = "SERVER",
}

vanguard.power = {
    ONLINE = "ONLINE",
    OFFLINE = "OFFLINE",
    STARTING = "STARTING",
    RESTORING = "RESTORING",
}

vanguard.health = {
    CLEAN = "CLEAN",
    SUSPICIOUS = "SUSPICIOUS",
    COMPROMISED = "COMPROMISED",
}

vanguard.task_state = {
    RUNNING = "RUNNING",
    COMPLETED = "COMPLETED",
    FAILED = "FAILED",
}

vanguard.policy = {
    DEFAULT_ALLOW = "DEFAULT_ALLOW",
    DEFAULT_DENY = "DEFAULT_DENY",
}

-- IDENTIFICATION -------------------------------------------------------------

-- Returns this Vanguard's agent ID.
---@return integer agent_id
function vanguard.self() end

-- Returns this Vanguard's deployment router node ID.
---@return integer router_id
function vanguard.router() end

-- Returns a map of active teammate agent IDs to their deployment router IDs.
-- Excludes self. Example: { [2] = 12, [3] = 18 }.
-- Iterate with pairs(), not ipairs(). Map iteration order is unspecified.
---@return table<integer, integer> agents
function vanguard.others() end

-- COMMUNICATION --------------------------------------------------------------

--[[
Messages are bounded JSON-like value trees: strings, booleans, finite numbers,
dense arrays, or string-keyed objects. Nil payloads, mixed-key tables, functions,
threads, userdata, cycles, and metatables are invalid. Values are copied at send
and delivery; agents never share Lua table references.

Direct messages address teammate agent IDs. Topics are exact case-sensitive
strings of 1..128 bytes, with no wildcards. No self-delivery, retained publications for future subscribers, replay, or
application-level acknowledgement is implied.

For valid arguments, tx() and pub() ALWAYS accept messages. There is no queue-full
failure, recipient-capacity rejection, or message-submission preflight function.
A missing handler is not a send error. At the next barrier, direct messages are
routed to the specified teammate and publications to the committed start-of-tick
subscriber set. Delivery order is sender agent ID then sender sequence.
Per-sender order is preserved. Sending to a known inactive teammate is accepted
and dropped. Nonexistent IDs and malformed payloads are programming errors and
raise normally. Sending to self is invalid.

At recipient delivery, no matching rx/sub handler means the message is dropped,
with no retry or sender notification. A recipient is not required to register a
handler. Publishing with no subscribers succeeds with recipient_count=0.
Sub/unsub changes in T become visible to other VMs at T+1. A new subscription
in T does not receive a publication from T. pub() recipient_count describes the
start-of-tick recipient set, not the count of handlers that eventually run.

Messages are delivered automatically. Each delivered message starts a new
host-managed coroutine INSIDE THE RECIPIENT ROUTER VM running its rx/sub handler.
Delivery is scheduled only after synchronization. No polling or
manual coroutine.resume is required. Messages with no handler at delivery are
discarded. The handler is captured at delivery; replacing/unregistering it does
not cancel already created coroutines. Unsubscribing discards local undelivered
topic messages; other VMs observe its membership change at the next barrier.

Coroutines are cooperative, not simultaneous Lua threads. They share the script's
Lua globals and closure state. The host schedules runnable coroutines in FIFO
order, using a fixed metered instruction quantum so a busy main script cannot
starve delivered handlers within that VM's tick allowance. Messages from tick T
become runnable in T+1, subject to handler capacity, never before the barrier. A yielding
handler can finish after a later message's handler; delivery order is not
completion order. Shared state may change across any yielding operation.

Handlers may call deploy(), await(), await_any(), wait(), tx(), and pub().
await()/await_any()/wait() suspend ONLY the calling coroutine. Other handlers,
and the main script continue within the same VM allowance. Router tasks advance
at synchronization barriers. A handler coroutine uses no task
slot itself; any task it deploys competes for the same five router slots.
Fuel consumed by all coroutines is charged to the same Vanguard agent.

A handler error is logged with sender and message ID and terminates only that
handler coroutine. It does not terminate the main script or other handlers,
retry the message, or cancel tasks already submitted by the failed handler.
If the host limits simultaneously active handlers, accepted messages remain
pending until a coroutine can start; this never causes a send rejection.
Coroutine limits and all message execution costs are public scenario constants.

Returning from the main script leaves its registered handlers active until the
simulation ends. With nothing runnable, the host idles the agent automatically.
No keepalive loop is required. The host automatically resumes its managed
coroutines; manually resuming/yielding them through coroutine.* is unsupported.

Metadata passed to handlers:
    { message_id = integer, sent_at = integer, delivered_at = integer }
]]

-- Sends a direct message. Valid submissions always succeed; no handler means drop.
-- Returns { message_id, sent_at }; invalid calls raise Lua errors.
---@param agent_id integer
---@param message any
---@return table acknowledgement
function vanguard.tx(agent_id, message) end

-- Replaces the direct handler; nil unregisters it. Returns true; invalid calls raise Lua errors.
-- Handler: function(sender_agent_id, message, metadata)
---@param handler function|nil
---@return boolean success
function vanguard.rx(handler) end

-- Queues delivery to start-of-tick topic subscribers, excluding self.
-- Returns { message_id, sent_at, recipient_count }; invalid calls raise Lua errors.
---@param topic string
---@param message any
---@return table acknowledgement
function vanguard.pub(topic, message) end

-- Registers/replaces one handler for the topic; no duplicate subscription.
-- Handler: function(sender_agent_id, topic, message, metadata)
-- Handler is registered locally before return. Cross-VM subscription membership
-- becomes visible at the next synchronization boundary.
---@param topic string
---@param handler function
---@return boolean success
function vanguard.sub(topic, handler) end

-- Removes the subscription. Idempotent. Undelivered topic messages are discarded.
-- Existing handler coroutines continue to completion.
---@param topic string
---@return boolean success
function vanguard.unsub(topic) end

-- FUEL AND TIME --------------------------------------------------------------

-- Cumulative VM, native-operation, communication, and admitted-task fuel.
-- Omit agent_id for self. Only self or teammates may be queried.
-- Teammate values are snapshots from the most recent synchronization.
---@param agent_id? integer
---@return integer consumed
function vanguard.fuel(agent_id) end

-- Synchronized tick T, constant for the entire current VM slice.
-- Does not expose enemy action counts.
---@return integer ticks
function vanguard.ticks() end

-- Yields until tick T + duration_ticks (integer >= 1). Uses no task slot.
-- Suspends only the calling coroutine; automatic delivery continues.
-- Returns true; invalid calls raise Lua errors.
---@param duration_ticks integer
---@return boolean success
function vanguard.wait(duration_ticks) end

-- Validates the descriptor and estimates its task cost without scheduling it.
-- Requires the same fields and permissions as deploy(), but needs no free slot
-- and acquires no target lock. A quote does not reserve capacity or target state.
-- Result: { fuel=integer, ticks=integer, exact=boolean, quoted_at=integer }
-- fuel excludes the cost() call itself. Costs cannot expose hidden state.
-- exact=false allows actual cost to differ; actual cost is fixed at admission.
---@param task table
---@return table estimate
function vanguard.cost(task) end

-- SENSE ----------------------------------------------------------------------

vanguard.sense = {
    SCAN = "SCAN",
    -- No target. Inspects only the deployment router's local topology.
    -- data = {
    --   observed_at = tick,
    --   hosts = {{id=node_id, type="WORKSTATION"|"SERVER"}, ...},
    --   routers = {neighbour_router_id, ...},
    -- }
    -- Neighbour routers are identified, not inspected. Lists are sorted by ID.

    TRAFFIC_SCAN = "TRAFFIC_SCAN",
    -- No target, window, or cursor.
    -- Returns ALL unique unordered endpoint pairs that successfully communicated
    -- through this router since simulation start, up to scan completion.
    -- data = { observed_at=tick, pairs={{a=node_id,b=node_id}, ...} }
    -- a < b; pairs sorted by a then b. A->B and B->A share one entry.
    -- Blocked attempts are excluded. Repeated communication adds no duplicates.
    -- Closed sessions, shutdown, and restoration never erase historical pairs.
    -- Same-LAN traffic bypassing the router is absent. Endpoints become known.
    -- Evidence of past communication is not evidence of infection/current access.

    HOST_SCAN = "HOST_SCAN",
    -- Required target: local host.
    -- data = { observed_at=tick, power="ONLINE",
    --          assessment="CLEAN"|"SUSPICIOUS"|"COMPROMISED" }
    -- Quick symptom-based assessment. CLEAN is not a guarantee of no infection.
    -- "In danger" is exposure inferred by contestants, not an infection state.

    CONNECTION_SCAN = "CONNECTION_SCAN",
    -- Required target: local host.
    -- data = { observed_at=tick, connections={
    --   {peer=node_id, direction="INBOUND"|"OUTBOUND", opened_at=tick}, ...
    -- } }
    -- Current logical sessions, including remote peers. Direction is relative
    -- to the scanned host and records who initiated the session. One record per
    -- session; multiple sessions to one peer are allowed. Sort by peer, opened_at,
    -- then direction. Peer IDs become known without granting remote control.

    FULL_HOST_SCAN = "FULL_HOST_SCAN",
    -- Required target: local host. More expensive than HOST_SCAN.
    -- data = { observed_at=tick, power="ONLINE", compromised=boolean,
    --          connections={...same format as CONNECTION_SCAN...} }
    -- Exact infection status and sessions from one completion-time snapshot.

    SHIELD_STATUS = "SHIELD_STATUS",
    -- No target: own router. Optional target: local host only.
    -- data = { observed_at=tick, policy="DEFAULT_ALLOW"|"DEFAULT_DENY",
    --          exceptions={sorted_peer_ids}, isolated=boolean }
    -- Router isolated is always false: router isolation is not exposed.
}

-- HOST RECOVERY --------------------------------------------------------------

vanguard.act = {
    SHUTDOWN = "SHUTDOWN",
    -- Required target: local host. At completion, power becomes OFFLINE and all
    -- sessions terminate. Services and spreading stop. Infection and shield
    -- settings persist. Already OFFLINE succeeds with changed=false.

    STARTUP = "STARTUP",
    -- Required target: local host. OFFLINE -> STARTING at the first barrier
    -- after admission, then ONLINE
    -- at completion. Services remain unavailable during startup. Infection and
    -- shield settings persist. Already ONLINE succeeds with changed=false.

    RESTORE = "RESTORE",
    -- Required target: local host. At the first barrier after admission,
    -- enters RESTORING, terminates
    -- sessions, and stops services/spreading. At completion, infection is removed
    -- and power becomes ONLINE. Defender shield policy and isolation persist.
    -- For duration=1, entry and completion share the same barrier. No VM observes
    -- the transitional state. Recovery commits before interval network activity.
    -- Accepts either ONLINE or OFFLINE hosts. Can be reinfected after completion.
    -- On failure after admission, remains OFFLINE; infection is not guaranteed
    -- cleared. Failed STARTUP likewise leaves the host OFFLINE.
}

-- SHIELD ---------------------------------------------------------------------

--[[
Router actions have NO target and apply only to the deployment router.
Host actions require target and apply only to a local workstation/server.
A peer can be any known endpoint ID, including a remote host, but cannot equal
an explicit host target or the implicit router target. Rules do not control peers.

DEFAULT_ALLOW: exception set contains BLOCKED peers.
DEFAULT_DENY: exception set contains ALLOWED peers.
Initial router and host policies are DEFAULT_ALLOW with no exceptions.
Initial host isolation is false.

Changing policy clears exceptions atomically; setting the same policy preserves
exceptions. Host isolation overrides policy and exceptions but preserves both.

Host rules match the other endpoint. Router rules match original source OR
original destination IDs: under DEFAULT_ALLOW neither may be blocked; under
DEFAULT_DENY BOTH must be allowed. Intermediate router IDs are not endpoints.
This prevents an allowed source from bypassing a blocked destination.

Traffic must pass all endpoint and transit filters on its path. Rules apply
bidirectionally to existing and new traffic. Newly forbidden sessions terminate.
Allowing traffic again does not recreate terminated sessions automatically.
Same-LAN traffic bypassing the router still passes the two hosts' filters.
]]

vanguard.shield = {
    SET_DEFAULT_ALLOW = "SET_DEFAULT_ALLOW",
    -- Own router: allow all endpoints except blocked exceptions.
    SET_DEFAULT_DENY = "SET_DEFAULT_DENY",
    -- Own router: allow only communication with both endpoints in exceptions.
    BLOCK_PEER = "BLOCK_PEER",
    -- Own router; requires peer. Add blocked/remove allowed exception.
    ALLOW_PEER = "ALLOW_PEER",
    -- Own router; requires peer. Remove blocked/add allowed exception.

    SET_DEFAULT_ALLOW_ON_HOST = "SET_DEFAULT_ALLOW_ON_HOST",
    -- Local target host: allow all peers except blocked exceptions.
    SET_DEFAULT_DENY_ON_HOST = "SET_DEFAULT_DENY_ON_HOST",
    -- Local target host: allow only peers in exceptions.
    BLOCK_PEER_ON_HOST = "BLOCK_PEER_ON_HOST",
    -- Local target host; requires peer. Add blocked/remove allowed exception.
    ALLOW_PEER_ON_HOST = "ALLOW_PEER_ON_HOST",
    -- Local target host; requires peer. Remove blocked/add allowed exception.
    ISOLATE_HOST = "ISOLATE_HOST",
    -- Local target host: block all data traffic while leaving power unchanged.
    RELEASE_HOST = "RELEASE_HOST",
    -- Local target host: lift isolation and resume saved filtering policy.
    -- Does not remove individual blocks or power on an offline host.
}

-- TASK DESCRIPTORS AND EXECUTION ---------------------------------------------

--[[
Descriptor (data only; no builder function is required):
    {
        action = action_constant,
        target = local_host_id, -- only where required/allowed
        peer = known_node_id,   -- only for BLOCK_PEER / ALLOW_PEER variants
    }

No other fields are accepted. Router-only actions reject target even if it is
this router's ID. SHIELD_STATUS accepts no target or a local host target.
Peer fields on non-peer actions are invalid. There is no window_ticks field.

Successful task result:
    {
        task_id = integer,
        action = string,
        target = local_host_id, -- omitted for implicit-router tasks
        router_id = deployment_router_id,
        started_at = tick,
        finished_at = tick,
        fuel_spent = integer,
        data = action_specific_data,
    }
For mutations, data = { changed = boolean }. Repeating an idempotent operation
can complete with changed=false; it still has a published cost and uses a slot.
Scans use the data shapes above; observed_at equals finished_at.
All tasks execute on the originating router, including tasks targeting local hosts.
A duration-D task admitted in T completes at boundary T+D unless it fails earlier.
]]

-- Validates and atomically admits a task on THIS router, returning its ID without
-- waiting for task completion. World effects commit at synchronization barriers.
-- The binding may first yield for sufficient tick allowance before admission.
-- Full slots raise TASK_SLOTS_FULL. No internal queue or implicit wait.
-- Locked target raises TARGET_BUSY. Other VMs cannot submit into these slots.
-- Only this VM can inspect its router's tasks.
---@param task table
---@return integer task_id
function vanguard.deploy(task) end

-- Snapshot: { capacity=5, running=integer, available=integer }.
-- Not a reservation. Does not consume a task slot.
---@return table slots
function vanguard.slots() end

-- Sorted IDs of currently RUNNING tasks on this router.
---@return integer[] task_ids
function vanguard.tasks() end

-- Snapshot:
-- { id, action, router_id, target?, state, started_at, finished_at?, fuel_spent,
--   failure? }
-- finished_at is absent while RUNNING. On FAILED, failure is a readable string
-- using the same reason prefix as Lua errors. Inspecting failure does not raise.
---@param task_id integer
---@return table status
function vanguard.status(task_id) end

-- Nonblocking. COMPLETED returns result. RUNNING raises TASK_NOT_FINISHED.
-- FAILED raises the recorded failure. Unknown ID raises UNKNOWN_TASK.
-- Repeated reads return equivalent independent copies; nothing is consumed.
---@param task_id integer
---@return table result
function vanguard.result(task_id) end

-- Yields until this task completes/fails; returns the same values as result().
-- Already terminal returns immediately. No extra slot. Only the calling coroutine
-- waits; automatic delivery and other coroutines continue.
---@param task_id integer
---@return table result
function vanguard.await(task_id) end

-- Yields until ANY task in the supplied nonempty list completes or fails.
-- Validates all IDs first. Duplicates are ignored. If multiple are terminal,
-- returns the lowest task ID. Failure of a task still satisfies the wait.
-- No extra slot and no result consumption; inspect outcome using result().
-- Only the calling coroutine waits; automatic message delivery continues.
-- Remove returned IDs from subsequent wait sets to avoid selecting them again.
---@param task_ids integer[]
---@return integer task_id
function vanguard.await_any(task_ids) end

-- PREFLIGHT AND READINESS -----------------------------------------------------
-- All checks are synchronous, use no task slot, and still consume call fuel.
-- They inspect this router's current committed snapshot and local reservations.
-- Malformed request data supplied to can_deploy() returns false,INVALID_ARGUMENT;
-- the checker does not throw for expected validation/precondition failures.
-- Internal VM failures/resource exhaustion are outside normal API guarantees.

-- Returns true if deploy(task) can currently admit this descriptor.
-- Otherwise returns false, reason_label (no human error object).
-- Checks fields, discovery, ownership, target type/state/locks, and free slots.
-- Does not reserve a slot or guarantee success after yielding or a tick boundary.
---@param task table
---@return boolean ready
---@return string|nil reason
function vanguard.can_deploy(task) end

-- Does a task ID exist in this router's task registry? False for invalid/foreign IDs.
-- Use before status(), result(), await(), or await_any() when IDs are uncertain.
---@param task_id integer
---@return boolean exists
function vanguard.has_task(task_id) end

-- True for COMPLETED or FAILED. False for RUNNING or unknown IDs.
-- A finished task may have FAILED: inspect status before result().
---@param task_id integer
---@return boolean finished
function vanguard.is_finished(task_id) end

-- True only for a COMPLETED task whose result can be read without a task error.
-- False for RUNNING, FAILED, or unknown IDs.
---@param task_id integer
---@return boolean ready
function vanguard.is_ready(task_id) end

-- ERRORS ---------------------------------------------------------------------

-- Stable reason labels used as Lua error-string prefixes and preflight reasons.
vanguard.error = {
    INVALID_ARGUMENT = "INVALID_ARGUMENT",
    UNKNOWN_ACTION = "UNKNOWN_ACTION",
    UNKNOWN_AGENT = "UNKNOWN_AGENT",
    UNKNOWN_NODE = "UNKNOWN_NODE",
    UNDISCOVERED = "UNDISCOVERED",
    OUT_OF_SCOPE = "OUT_OF_SCOPE",
    INVALID_TARGET_TYPE = "INVALID_TARGET_TYPE",
    OFFLINE = "OFFLINE",
    TARGET_BUSY = "TARGET_BUSY",
    TASK_SLOTS_FULL = "TASK_SLOTS_FULL",
    UNKNOWN_TASK = "UNKNOWN_TASK",
    TASK_NOT_FINISHED = "TASK_NOT_FINISHED",
    PAYLOAD_TOO_LARGE = "PAYLOAD_TOO_LARGE",
    LIMIT_EXCEEDED = "LIMIT_EXCEEDED",
}

--[[
EXAMPLE: ROUTER MAP
for agent_id, router_id in pairs(vanguard.others()) do
    print(agent_id, router_id)
end

EXAMPLE: AUTOMATIC MESSAGE HANDLING
vanguard.rx(function(sender, message)
    if message.type == "SCAN_HOST" then
        local task = {action=vanguard.sense.HOST_SCAN, target=message.node}
        local ready, reason = vanguard.can_deploy(task)
        if not ready then
            vanguard.tx(sender, {type="SCAN_SKIPPED", reason=reason})
            return
        end
        local id = vanguard.deploy(task)
        vanguard.await_any({id}) -- Wait for terminal state without raising on failure.
        local state = vanguard.status(id)
        if state.state == vanguard.task_state.COMPLETED then
            vanguard.tx(sender, {type="SCAN_RESULT", result=vanguard.result(id)})
        else
            vanguard.tx(sender, {type="SCAN_FAILED", reason=state.failure})
        end
    end
end)

-- No matching receiver handler? This valid send still succeeds.
-- vanguard.tx(teammate_id, {text="Hello"})

EXAMPLE: CHECK BEFORE SUBMISSION
local task = {action=vanguard.sense.TRAFFIC_SCAN}
if vanguard.can_deploy(task) then
    local id = vanguard.deploy(task)
    vanguard.await_any({id})
    if vanguard.status(id).state == vanguard.task_state.COMPLETED then
        local result = vanguard.result(id)
        print(#result.data.pairs)
    end
end

-- Preflight is advisory. For unexpected errors use normal Lua pcall:
local ok, result_or_error = pcall(function()
    return vanguard.deploy({action=vanguard.sense.SCAN})
end)
if not ok then print(result_or_error) end
]]

-- Scenario extension: canonical defaults are supplied by the Rust action registry.
-- These are simulation durations, not computation fuel charges.
vanguard.tick_cost = {
    SCAN = 2,
    TRAFFIC_SCAN = 4,
    HOST_SCAN = 3,
    CONNECTION_SCAN = 3,
    FULL_HOST_SCAN = 8,
    SHIELD_STATUS = 1,
    SHUTDOWN = 1,
    STARTUP = 3,
    RESTORE = 20,
    SET_DEFAULT_ALLOW = 1,
    SET_DEFAULT_DENY = 1,
    BLOCK_PEER = 1,
    ALLOW_PEER = 1,
    SET_DEFAULT_ALLOW_ON_HOST = 1,
    SET_DEFAULT_DENY_ON_HOST = 1,
    BLOCK_PEER_ON_HOST = 1,
    ALLOW_PEER_ON_HOST = 1,
    ISOLATE_HOST = 1,
    RELEASE_HOST = 1,
}

return vanguard
