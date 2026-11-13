# Simulation decisions

## Stop on verified victory (2026-11-12)

This supersedes the fixed-deadline continuation rule below. Scored matches end at
the first barrier that completes the configured full-recovery window. Zero
compromised hosts alone does not finish a run: exposure, every server serving
clients and the required-service threshold still apply. The runtime enters
FINISHED, the outcome is VICTORY and further step calls leave the world unchanged.
The core also rejects barriers after victory. Restart resets all accounting.

To preserve fixed-horizon scoring, remaining deadline ticks are credited at the
verified final state's service and clean-host fractions. Partial service keeps
its actual fraction; final operational recovery keeps its existing definition.
No virtual world/agent/enemy ticks execute. Early-defeat padding remains zero.

## Competition mailbox and optional drawings (2026-11-11)

VanMail extends the Outlook mailbox to 43 character-led emails and thirteen
folders. The first message is a generated category directory with seven-hour
rules and a 30-minute starter path. Organizer examples may be copied/adapted;
external LLMs are for research, public-example explanations and topology data,
not generated competition agent code or uploads of competitor source. Internal
build/install instructions live outside competitor mail. The offline build is
one bundled HTML file using hash navigation and no external assets.

`--dbg` is a CLI-only capability, retained by RunFiles across Restart. It exposes
seven write-only `dbgguard` calls, independently namespaced by agent; arbitrary
claims do not grant simulation discovery or control. Calls pay normal VM/native
and serialization fuel. Store updates are bounded and transactional, transported
as a top-level debug snapshot field, and omitted from semantic hash content
(their actual fuel/scheduler effects still count). Drawings clear on fresh runs.
The dashboard compares two selected views, paginating 80 nodes and labelling
agent-authored claims separately from truth/evidence.

Basic, Coordinated and Advanced are portable baseline policies, not omniscient
solutions. Shared patrol state machines overlap jobs within five slots. Confirmed
source messages guide transit filtering only after discovery; cure reports remove
those blocks. The generator publishes local required services explicitly. Tested
scale results do not imply wins on all legal topologies, crowded single LANs,
deadlines or initial infections. Service accounting now precomputes failed
endpoints/serving servers in one pass, preserving scoring semantics at 10k nodes.

## Scoring and explorer revision (2026-11-11)

The latest scoring request supersedes the former no-scoring default. Finite TOML
networks containing workstations and servers default to scored missions; other
networks default to practice. `[mission]` publishes explicit weighted required
workstation/server pairs, or expands to all such pairs. All servers are critical.
Scoring uses 600 service-time, 250 clean-host-time and 150 final operational-host
points, with fixed deadline denominators. Recovery requires no compromise or
exposure, every server serving clients, and 90% of weighted services for 14 ticks.
Recovery is revocable and does not stop the run. The user selected a 30-tick total
service-outage grace, permitting a 20-tick restore on a single-server network.
Defeat stops at a real barrier, pads the remaining horizon with zero credit and
removes final recovery credit. Probes are read-only routing checks, not sessions.
Integer millionth fractions accumulate deterministically; points truncate to two
decimals. Practice does not apply mission endings. `scoring.md` is the concise rulebook.

The latest dashboard reference is `operation_vanguard_dashboard(1).html`.
Its search/hierarchy/aggregate overview, paginated inventory, LAN detail, server
shortcuts, situation/score brief and telemetry dock replace the earlier rail/map
workspace. Icons use Lucide; true state remains separate from agent observations.
The browser still receives canonical full snapshots/top-level deltas and searches
them locally. Bounded rendering is verified with a 2,048-host fixture, not a claim
of server-side indexed search or large-world simulation throughput.

Ratatui, Crossterm and the separate legacy runner are removed. Startup prints
ASCII Vanguard and the actual web URL after binding, including ephemeral ports.


## Current startup and workspace (2026-11-11)

The latest user requirements supersede the earlier default-script/manifest and
four-pane gallery arrangement below. Startup requires `--script agent.lua
--network network.toml`. The network is explicit version-1 TOML; there is no
default agent or per-router program selection. Every router runs the supplied
entrypoint in a private VM. The entrypoint's parent is the staged dependency root.
Network files contain router/LAN/host lists, router connections and optional
simulation settings only. No initial compromised flag means clean; initial
compromise remains restricted to workstations. Restart reloads both CLI inputs.

The dashboard now follows the supplied command-workspace prototype: map-first
layout, right inspector, telemetry dock and rail shortcuts. `/components` was
removed. StepTicks admits 1–1000 ordinary barriers from READY/PAUSED and remains
paused unless the run finishes. It is serialized/idempotent like other controls.
`vandocs/` provides the standalone statically exported Fumadocs handbook.

Earlier notes below are historical design context where they conflict with this
section.

The acceptance contract is `src/api.lua` v1.3. It is an interface document, not
loaded as a simulator. Only the requested `tick_cost` table was appended. Runtime
constants come from the canonical Rust action registry. Durations are simulation
ticks and are unrelated to instruction fuel.

## Resolved defaults

| Gap | Implemented resolution |
| --- | --- |
| Five views/four panes | Simulation spans the left rows, Vanguard and Node sit above a shared Stats/Jobs pane; narrow screens stack them. |
| Deployment | Loopback Rocket process embeds the statically built Svelte frontend. No Svelte runtime service. |
| Server ownership | Each server has one authoritative router LAN, including visually central servers. |
| Attack concurrency | At most two sessions per compromised source, refilled when slots become available. |
| Targets | Seeded selection from sorted distinct ONLINE, clean, unexposed workstation/server candidates with permitted routes. |
| Exposure | Fourteen consecutive successful intervals; interruption clears only pre-compromise exposure and logs the reset. |
| Display health | Clean=5, compromised=0, exposed=ceil(5*(14-age)/14), clamped 1..5; routers=N/A. |
| Quick scan | Deterministic CLEAN/SUSPICIOUS/COMPROMISED baseline. No hidden stealth/noise. |
| Counts | Infection categories partition hosts only; immune routers counted separately. |
| Enemy turns | One enemy update per barrier, separate connection/attempt counters. |
| End condition | Pause or configured tick limit; no hidden victory condition or score. |
| Job click | Opens a detail dialog with timing, progress, result/failure, fuel and related events. |

## Determinism and topology

- Static topology builds sorted-neighbour BFS parent tables once (O(R²) storage).
  Route queries preserve the original directional shortest-path tie breaks. Filter,
  isolation and power checks stay live; blocked paths never silently reroute.
- Hosts are indexed by router; running tasks by router; mutation locks by target;
  exposure by target. Completed jobs stay queryable without participating in slot,
  lock or barrier-running-job scans. Indexes are derived state, not replay content.
- Attack candidate enumeration occurs only when a source has a vacant attack slot.
  Active targets are grouped by source; sorted target order and PRNG consumption
  remain unchanged. Service sets of at least 8,192 pairs evaluate in up to eight
  independent read-only chunks, joined in source order before integer accounting.
- Parallel router VMs in fixed, sorted cohorts of eight and a shared barrier.
  Lua instruction execution overlaps on worker threads; shared-effect native calls
  and scheduler writes take deterministic round-robin turns. Read-only calls see
  committed state plus the calling router's own reservations. Cohort size is
  independent of hardware, so CPU count and OS scheduling do not affect outcomes.
  Every VM sees committed W[T]; admissions and locks are local reservations,
  not world mutations. Each VM retains its own fuel, memory and coroutine FIFO.
- Workers join before the world/enemy/task/message barrier, snapshots or restart.
  At most eight VM threads exist at once, even for a 1,000-router configuration.
  A returned/errored VM leaves the cohort's turn rotation without blocking peers.
- Scheduler identity is recorded in snapshots and semantic hashes. Native calls
  now interleave across routers, so task/message ID allocation and replay hashes
  can differ from the historical sequential scheduler; same-version runs remain
  deterministic. This does not change task durations or infection intervals.
- Sorted router/agent IDs and submission order. The enemy uses its own seeded
  PRNG, not Lua or UI randomness. The PRNG state is part of simulation metadata.
- The selected shortest physical router path uses ID tie breaks. Filters are
  applied to that path without silent rerouting. Same-LAN communication bypasses
  the router filter and still obeys both endpoint filters.
- Successful transit intervals add canonical original endpoint pairs to router
  history. Permissions reopening does not itself recreate a session.
- Initial infection is limited to workstations. Routers remain immune. Power,
  isolation, assessment, exposure, and established infection are separate fields.

## Lua hosting and costs

The binding is pinned to **mlua 0.12.2**, vendored **Lua 5.5**. A test proves
suspension through `pcall` before native effects and exactly-once resumption.
Every instruction-hook event costs one fuel. A quantum bounds FIFO coroutine
slices; all coroutines and native bindings share the router's tick allowance.
Waiting uses no task slot, and unfinished tasks progress at each barrier even
when the main coroutine has returned.

Native calls pay the configured base call cost. Successful admissions additionally
pay the configured task admission cost once. Communication pays serialized
payload bytes; printing pays bounded structured-output bytes; string operations
pay input bytes and repetitions additionally pay output bytes; staged module
compilation pays source bytes. A charge that does not fit the remaining allowance
yields before effects and is retried/revalidated on a later tick. Operations above
a full-tick atomic allowance fail with LIMIT_EXCEEDED. Payload size, source size,
handler count and VM memory are bounded. Native operations are **not** claimed to
be covered by the instruction hook.

The sandbox has no filesystem, processes, sockets, debug, dynamic loader, wall
clock or raw coroutine controls. `require` reads a consistent staged source set,
never live files. Table sequence helpers/sort are pure Lua and metered. Scalar
key iteration is sorted; unsupported pointer-key iteration is rejected. Safe
agent metatables are copied/sealed; GC finalizers, close callbacks and weak-table
mode are rejected so allocator/collector timing cannot schedule agent code. Safe
string helpers are bounded; pattern matching/format/dump and nondeterministic
math.random are not exposed. `string.find` performs literal matching. Structured
print avoids metamethods, represents cycles/unsupported values explicitly, and
truncates by fixed depth/entry/string limits. These standard-library restrictions
are part of the supported sandbox, not alternate Vanguard API signatures.

## Protocol, lifecycle, and resources

- The dashboard opts into `GET /api/ws?records=true`. Its update envelopes carry
  `patch` rather than `delta`, with the same run/sequence/base-sequence checks.
  Patches replace scalars (`set`), recurse object `fields` with `removed` keys,
  append unchanged-prefix arrays (`append`), sparsely patch fixed-position array
  records (`items`), or update/evict stable-ID arrays (`key`, `changes`, `removed`).
  Full replacements handle arbitrary shape/order changes. Applying a patch must
  reproduce the next full projection exactly. Unchanged object identities survive
  the immutable reducer and normalization. Legacy `/api/ws` retains field deltas.
- Headless execution collects boundary hashes only under `--replay`, reuses its
  final recorded hash, and constructs the final dashboard snapshot only for replay
  output. Semantic JSON is streamed into SHA-256 through a 64 KiB buffer; pretty
  replay output streams to a buffered file with explicit flush/error propagation.
  The JSON representation and replay hashes are unchanged. VM callbacks/workers
  copy scalar limits rather than cloning/retaining topology and mission services.
- Session lookup is indexed by directional endpoint pair and host. Removal updates
  both indexes for filter interruption, recovery and infection. Infection builds
  one sorted clean/online/unisolated target inventory per boundary, retaining
  per-source permission checks, claimed-target exclusion, order and RNG consumption.
- JSON WebSocket `/api/ws`, HTTP snapshot and command fallbacks. Protocol v1
  envelopes contain run ID, committed tick, sequence and correlated request ID.
- Tick updates carry changed top-level projection fields plus `base_seq`.
  Unchanged topology and collections are omitted; changed arrays replace their
  field atomically. One canonical projection and merge/normalize reducer serve
  live updates and resynchronization. The bounded semantic event window remains
  available inside the snapshot. This is field-level delta encoding, not
  per-element compression of large changing arrays.
- Subscribe-before-snapshot with a sequence watermark closes the connection
  race. A lagged socket gets a fresh projection. Clients resync on sequence gaps
  and ignore stale-run updates. Disconnect does not change engine state.
- Up to 512 request IDs are idempotently cached for this server process, including
  across restart, within a 16 MiB serialized-reply budget. The newest reply is
  retained even if it alone exceeds that budget. IDs must not be reused for
  different commands; older entries may be evicted as either bound is reached.
- Continuous playback publishes projections at most 10 Hz, independently of tick
  pacing. Control replies, terminal transitions and explicit snapshot commands are
  immediate. Socket history holds eight serialized envelopes shared across clients;
  lagged clients resynchronize rather than retaining an unbounded backlog.
- The browser reduces every wire sequence into an immutable canonical projection,
  coalescing live paints to 100 ms. Normalization reuses unchanged arrays; snapshots
  are not deeply proxied. Outcome/control changes and acknowledgements flush
  immediately. Local search, selection, scrolling and resizing remain interactive.
- Closed dialogs mount no body content. JSON children mount only when expanded,
  with 100 entries/page. Jobs and inspector connections show 100 records/page;
  observations show 50/page. All records remain accessible through pagination.
  LAN matching uses maps/sets instead of repeated linear membership searches.
  Graph icons are cached and unchanged element data/positions are preserved.
- Pause is serialized by the simulation owner after any in-flight barrier.
  There is no stepping while paused. Resume keeps the same VMs. Restart stages
  configuration and all allowed Lua files before swapping in fresh VMs and a new
  world. Failure leaves the previous run paused and readable.
- One source file is at most 1 MiB, one root at most 256 Lua files, with no
  symlinks. Only manifest-relative files and sanitized module names are allowed.
- Logs have a bounded retained window; jobs/results persist for the run. Accepted
  pending messages are not rejected by recipient capacity. More than 100,000
  pending recipient deliveries explicitly stops the run with LIMIT_EXCEEDED.
- No autonomous defender actions are built into the enemy/core. The example
  defender is ordinary Lua. No scoring formula was added.

## Verification and limits

Tests distinguish semantic state/event hashes from diagnostic VM allocator bytes
and presentation speed. The dashboard is an operator truth view; agent discovery
is separate recorded data and never auto-filled from truth. Cross-process/browser
rendering cannot advance the simulation. See the root README for commands and
the final verification report for checks actually executed.
