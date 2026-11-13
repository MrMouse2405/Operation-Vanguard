# Operation Vanguard

Local network-defence simulation: an authoritative Rust engine, one private Lua
VM per router, and a statically built Svelte/Cytoscape command workspace.

## Run

Requires Rust and [Bun](https://bun.sh). Cargo installs locked UI dependencies,
builds the static frontend and embeds it. The resulting binary needs no frontend
runtime. Lua files and the network file stay on disk.

```sh
cargo run -- --script vanguard/agents/defender.lua --network network.toml
```

Open **http://127.0.0.1:8000** and press Start. The CLI requires both named inputs:
there is no default agent and no per-router script selection.
Startup prints the Vanguard ASCII banner and actual dashboard URL. No terminal UI
is used; router fuel and memory are visible in the dashboard's Statistics tab.

The explorer's **Topology** and **Table** views share the same scope. At
**Network → All LANs**, Table lists LANs; selecting a LAN row lists its nodes.
Drag pane dividers to resize columns, telemetry, and debug comparisons, or focus
a divider and use arrow keys. Column widths and telemetry height persist locally.
Match score is the default right panel; a selected node's X returns to that score.

```sh
cargo install --path .
vanguard --script agent.lua --network network.toml
```

Maintainer Git-install and distribution instructions: [docs/install-from-git.md](docs/install-from-git.md).

**Every router runs the same supplied code in a different VM.** Globals, module
caches, handlers, memory and fuel are private. Hosts never run agent VMs.

Router VMs execute Lua concurrently in fixed groups of up to **eight worker
threads**. Shared-effect native calls take deterministic round-robin turns within each group;
the world advances only after every router finishes its tick allowance or waits.
This preserves reproducible outcomes independently of OS scheduling and CPU
count. Fuel and task durations still use simulation ticks, not wall-clock time.
Native-call-heavy workloads and the network/enemy barrier may limit speedups.

Use `--port 8080` for another port or `--port 0` for an available port. The actual
URL is printed on startup. The process binds to loopback by default.

### Agents

- `vanguard/agents/defender.lua`: discovers, coordinates, scans and isolates/restores.
- `vanguard/agents/ostrich.lua`: deliberately does nothing and returns; the host
  idles its VMs while the network/enemy continues advancing.
- `discovery.lua`, `messaging.lua`, `pubsub.lua`: focused API examples.
- `team.lua`: a controlled dependency used by the defender.
- `basic-defender.lua`, `coordinated-defender.lua`, `advanced-defender.lua`: rolling
  five-slot patrols, sharing `patrol.lua`; coordinated variants share source reports
  and manage transit blocks. Advanced prioritizes servers.
- `vanguard/agents/examples/`: runnable sensing, shielding, recovery, solo, warning,
  leader and optional debug-reconstruction exercises.
- `vanguard/networks/`: tutorial/three-LAN topologies and an explicit TOML generator.
- [`progressive/`](progressive/README.md): ten complete networks from 10 to 10,000
  total nodes plus a 100,000-node / 400-LAN extension, with a 500-node/LAN cap.
  Includes a containment-first reference agent verified to win all eleven fixtures.

The entrypoint's parent is its controlled script root. `require('team')` uses
staged `team.lua`; all Lua dependencies are staged consistently before startup.
Keep agents in a small dedicated directory. Returning leaves registered handlers
and admitted tasks alive. No keepalive loop is needed.

## Network file

`network.toml` explicitly lists four routers, four LANs, 16 workstations, two
servers, one compromised workstation and four router links. No nodes are generated
by the CLI. Edit it by hand or generate the same simple format with a script:

```toml
version = 1
routers = [{ id = 1 }, { id = 2 }]
lans = [{ id = 10, router = 1 }, { id = 20, router = 2 }]
workstations = [{ id = 1001, lan = 10, compromised = true }]
servers = [{ id = 2001, lan = 20 }]
connections = [{ from = 1, to = 2 }]

[simulation]
seed = 1
tick_ms = 100
tick_limit = 1000 # 0 means unlimited
```

Connections are undirected **router links**; host attachment follows explicit LAN
membership. Same-LAN communication bypasses router filters but still obeys both
host filters. Each router owns exactly one LAN; hosts have exactly one owner.
Node IDs are globally unique; LAN IDs are a separate namespace. The router graph
must be connected. Invalid ownership, duplicate IDs/edges, unknown fields and
initial server/router compromise are rejected with actionable errors.

Only workstations may start compromised. Both workstations and servers can become
compromised through attacks. Routers are always immune. Omitted `compromised`
means false: no hidden initial infection is added.

Optional `[simulation]` settings: `seed`, `tick_ms`, `tick_limit`, `fuel_per_tick`,
`instruction_quantum`, `memory_limit`, `max_payload_bytes`, `max_handlers`,
`native_call_cost`, `task_admission_cost`, `infection_intervals`, `attack_slots`.
See `vandocs/content/mail/network.md` for network rules. Neither this
file nor simulation settings accept script selection.

## Command workspace

The dashboard follows the supplied network-explorer prototype: global search,
LAN hierarchy, aggregate health map, paginated node table, server inventory,
right-hand Match score/node inspector, and collapsible telemetry dock. LAN detail
renders up to 80 hosts per page; the overview shows aggregates rather than all
physical edges. The component-gallery route has been removed.

Large-network views use indexed host/LAN lookup, immutable snapshots, lazy JSON
details and bounded lists. Jobs and connections page in groups of 100; observations
page in groups of 50, and expanded JSON collections page in groups of 100. Closed
dialogs do not render their contents. Live projections update at most 10 times per
second without slowing or skipping simulation ticks; controls publish immediately.

**Globe**, beside Topology and Table, shows a dotted Earth with directional live
communications and Vanguard task arcs. Blue arrows are running router tasks;
amber is active exposure, red a compromised endpoint, and emerald another live
host connection. LAN locations are stable, **illustrative geography**. Drag or use
arrow keys to orbit, use `+` / `-` to zoom, filter by LAN/activity, and inspect
endpoints in the paginated flow directory. Geometry is local: no map service or
internet connection is required. The drawing is bounded to 160 flows (both task
and connection capacity reserved); all flows remain available in the directory.
Motion pauses with the match and respects reduced-motion preferences. Topology
uses matching coloured, moving directional task/session arrows. Required-service
dependencies are not presented as live sessions.

**Jobs** now combines Jobs and Tasks, with All / Running / Completed / Failed
counts and filters. Running tasks sort first; select a job to inspect its result,
failure, fuel and associated events.

Live browser updates use record-level patches and append-only observation/event
updates rather than replacing whole changed inventories. Reconnect still uses a
full snapshot. Headless execution without `--replay` skips intermediate snapshot
hashes; the final printed hash remains identical to replay mode.

- **Pause** acknowledges a stable barrier; no world/task/message progress follows.
- **Resume** preserves VMs and world state.
- **+1 tick / Forward** execute every normal backend barrier and remain paused,
  or enter FINISHED at the tick limit. They never just increment a display counter.
- **Restart** rereads the supplied network and script/dependencies, stages fresh
  VMs/world, and returns to READY with a new run ID. Failure leaves the old run
  paused and reviewable.
- Filters only affect presentation. Discovered state contains that agent's actual
  recorded observations, never operator truth substituted for missing data.
- WebSocket snapshots/field deltas support reconnect, resync and stale-run
  rejection. Client disconnect does not change simulation state.
- **s/tick** sets a minimum continuous-playback tick duration, including engine
  work. A faster tick waits only for the remaining time; a slower tick gets no
  extra delay. **0** runs as fast as the engine can. Initial pacing comes from
  `simulation.tick_ms` (default 0); the supplied network explicitly uses 100 ms.
  Changes are server-owned, visible to other browsers, and retained on Restart.
  Manual +1 / Forward stepping executes the requested barriers directly.
- Sidebar host selection opens and locates the host in Topology. Connected peers
  remain visible despite search filters or LAN scope (up to 80 extra peers, with
  all connections available in the inspector). Arrows show active connections
  automatically; there is no Physical/Sessions switch.
- The clickable **Ctrl K / ⌘ K** badge and either keyboard shortcut focus/select
  host search. Focus frames the whole search field, including its icon.

## Documentation inbox: VanMail

`vandocs/` is a standalone **SvelteKit static mail app**, using shadcn-svelte
controls and Lucide icons. Its Outlook-style inbox teaches the complete handbook
through 43 character-led emails from senders such as Capt. Network and Lt. Task.
The learning path covers all 23 Lua API functions, all 19 task actions,
enums/errors, infection rules, scheduling, messaging, network files and sandbox.
Browse topic folders, search every email, star useful lessons, or open an email
in a new tab. Read/unread and stars are saved locally in your browser.
The briefing indexes every email by category, sets the seven-hour competition
rules and offers a 30-minute starter path. Tutorials, Interface guide, Lua guide,
Example agents, Network Samples and Debug workshop support the reference folders.
ModeWatcher persists light/dark preference; fenced blocks highlight and copy their
exact source. Runnable attachments are included directly from the files under
`vanguard/`, so the handbook and tested examples cannot drift independently.

```sh
cd vandocs
bun install --frozen-lockfile
bun run dev             # http://localhost:3001
bun run check:coverage
bun run build           # one self-contained out/VanMail.html
bun run preview         # serve the export on port 3001
```

Share **vandocs/out/VanMail.html** as one file: it works offline from `file://`,
with all mail, CSS, JS, highlighting and downloadable examples embedded. The build
produces only that file in `out/`. Email content is
in `vandocs/content/mail/*.md`, with senders and learning order in
`vandocs/src/lib/mail.ts`. Existing `/docs/.../` links remain valid.
The interface contract
remains `src/api.lua` v1.3 plus the requested `tick_cost` table.

### Optional agent drawings

Run with `--dbg` to expose `dbgguard.view`, `node`, `edge`, `note`, `clear`,
`remove_node` and `remove_edge`. Each router can publish named drawings and copied
metadata without reading engine truth. The dashboard compares two perspectives
side by side, with 80-node pages and expandable notes/records. Drawings are
namespaced per agent, reset on restart, and included in headless snapshots.
Calls use normal fuel plus payload charges; instrumentation can affect timing.
See the Debug workshop in VanMail for signatures, limits and reconstruction agents.

## Scoring

See [scoring.md](scoring.md). Scored matches award **600 service + 250 security +
150 recovery** points. Recovery needs eradication and working services sustained
for 14 ticks; verified recovery immediately ends the match in victory. Zero working
required connections for 30 ticks is defeat. Unresolved matches reach the deadline.
An early win credits unused ticks at its final service/clean fractions; an early
defeat credits zero for those ticks. The dashboard and replay expose the
authoritative score breakdown. Finite workstation/server networks are scored by
default; `[mission]` supports explicit service pairs, windows, or `enabled=false`
for practice. The concise competitor manual adds strategy and worked Lua examples.

## Headless replay

```sh
cargo run -- --script vanguard/agents/ostrich.lua --network network.toml \
  --headless --ticks 100 --replay replay.json
```

Output contains each committed tick's semantic SHA-256 hash and a final snapshot.
Presentation pacing and allocator diagnostics are excluded; RNG, fuel, scheduler
ordering and semantic events are included. It is not a resumable Lua-heap checkpoint.
Use `--print-config` with both required inputs to inspect effective network settings.

## Development and checks

```sh
cargo run -- --script vanguard/agents/defender.lua --network network.toml
bun --cwd ui run dev       # another terminal; proxies /api to Rocket :8000

cargo test
cargo fmt --check          # requires Rust formatting tools
bun --cwd ui run check
bun --cwd ui run test:e2e
bun --cwd vandocs run check:coverage
bun --cwd vandocs run build
bun --cwd vandocs run check
bun --cwd vandocs run test:e2e
```

Playwright requires Chromium (`bun --cwd ui x playwright install chromium`) or
`PLAYWRIGHT_CHROMIUM_EXECUTABLE=/absolute/path/to/chrome`. Dashboard tests launch
real Rocket on 18124; docs tests serve the static export on 18125.

Backend-only development can set `VANGUARD_SKIP_UI_BUILD=1` after a successful UI
build. Normal Cargo builds embed the current static dashboard.

## Troubleshooting

- `cargo run --script ...` is parsed by Cargo. Use `cargo run -- --script ... --network ...`.
- A bad Lua or network edit leaves Restart paused with its staging error. Fix and retry.
- Missing discovered details: choose a Vanguard perspective and allow its scans
  to complete; operator truth is never used as a fallback.
- Native-operation rejection: inspect effective fuel/payload limits. Charges are
  deterministic size-based costs, not native CPU instruction counts. Allowed
  libraries are restricted; `string.find` is literal, not Lua patterns.

Implementation decisions and verification notes are under `docs/`.
