# Verification — 2026-11-11

## Executed checks

### Record updates, replay processing and globe — 2026-11-12

- **54 Rust tests pass**. New lossless patch reconstruction cases cover object
  changes/removal, observation append, record eviction, reordered arrays and
  restart shapes; a single changed host among 10,000 produces a patch under 200
  bytes. Session-index tests verify directional reuse, opposite directions,
  interruption cleanup and reopening with a fresh ID.
- **12 Chromium tests pass**, including globe rendering/animation, LAN/activity
  filters, endpoint inspection, keyboard camera controls, paused-frame stability,
  mobile overflow and the combined Jobs lifecycle filters. A nested wire patch
  completes a task and appends an observation without replacing inventories.
  Existing reconnect, stepping, score, debug, pane and 10,000-node regressions pass.
  Svelte check: **0 errors, 0 warnings**. Release build passed. One concurrent
  debug/release UI build collided in `.svelte-kit/output`; rerunning the release
  build after the browser build completed passed. UI detector: `[]`.
- Headless plain/replay comparison uses the prior optimized release (`b45d004`)
  and this release, `ostrich.lua`, progressive level 10 and 20 barriers:

  | Mode | Before | After | Maximum RSS before / after |
  | --- | ---: | ---: | ---: |
  | No replay | 3.748 s | 1.133 s (~3.3×) | 290,516 / 199,884 KiB |
  | Replay | 3.746 s | 3.577 s | 290,468 / 203,996 KiB |

  These single-run, same-machine measurements include startup and final hashing;
  replay also includes snapshot/output serialization. Plain mode skips unused
  intermediate hashes. Scalar VM limits remove retained topology/service clones.
  All **21 outbreak replay hashes** and both plain final hashes match the prior
  release. The final-network control still wins **T45 / 999.91** with all **46
  hashes** matching the prior parallel engine and zero runtime errors.
- On the real final-test network with Basic Defender, simultaneous legacy and
  record-patch sockets observed the same nine simulation publications. Legacy
  updates totalled **47,573,406 bytes**; record updates totalled **1,282,216 bytes**
  (**97.3% less**). Initial/full snapshots and command acknowledgements are excluded;
  tiny scalar-only controls have slightly more patch framing overhead. Reduction
  varies with actual changes; it is not a universal compression ratio.
- The actual 10,000-node globe had **906 mounted elements**, no horizontal overflow
  at 390px, and no browser errors. Desktop and phone captures were reviewed.
  Globe drawing is bounded to 160 arcs, reserves host-session capacity alongside
  task capacity, and exposes every filtered flow via eight-row pages. Dot Earth
  geometry is hand-authored/local with clearly illustrative LAN locations.
  Directional blue task arrows and amber/red/emerald sessions use actual engine
  records. Topology shows the same activity colours and animated dashed arrows.
  Animation stops on pause/reduced-motion and suspends drawing offscreen/hidden.
- Evidence remains ignored under `.impeccable/review/globe-*`; persistent tests are
  `src/projection.rs`, `src/sim_tests.rs` and `ui/tests/globe.spec.ts`.

### Large-network engine and dashboard optimization — 2026-11-12

- Cached static sorted-neighbour BFS trees with O(R²) parent storage. Live endpoint
  power/isolation/filters and transit policies still decide permission; cached
  routes do not reroute around blocked paths. Indexed LAN hosts, running jobs,
  mutation locks and active exposure; skipped attack-candidate enumeration when a
  source already has its slots filled. Large required-service sets evaluate in
  ordered, read-only parallel chunks before the usual integer score accounting.
- `VANGUARD_SKIP_UI_BUILD=1 cargo test --quiet`: **51 passed**, including a reference
  BFS comparison, live-policy checks, job-index lifecycle, parallel/serial service
  flag equivalence and first-boundary publication during slow playback.
- Release replay compatibility: all **46 hashes** match the previous parallel
  engine on the final fixture's T45 / 999.91 control victory; all **21 hashes**
  match on a 20-tick uncontrolled outbreak on progressive level 10. No Lua errors.
- Same-machine five-tick headless benchmark, including replay hashing/writing,
  `ostrich.lua` on progressive level 10 (10,000 nodes, 240 initial infections):
  **33.835 s before → 1.376 s after (~24.6×)**. Hashes match at every boundary.
  This is a fixture-specific measurement, not a universal speedup claim. The
  optimized control's complete scored replay takes 7.01 s in the recorded run.
- On the real final-test network, Chromium 1440×1000 initial overview:
  **4,322.6 ms → 285.2 ms**, **936,228 → 1,422 DOM elements**, and sampled JS heap
  **1,378,581,084 → 13,885,036 bytes**. Host search takes 23.1 ms in the optimized
  capture. Controls, LAN selection, inspector and live playback produce no browser
  errors. Desktop overview retains its previous visual hierarchy.
- Fixed the dominant hidden-DOM cost: closed dialogs now mount no body; JSON
  descendants mount on expansion, 100 entries/page. Jobs/connections paginate at
  100; observations at 50. Indexed matching replaces quadratic LAN membership
  scans. Raw immutable snapshots and cached normalization avoid deep proxying and
  rebuilding unchanged arrays. Graph icons/data updates are reused; selecting an
  existing-page node no longer resets the viewport.
- Simulation barriers remain authoritative. Automatic projections publish at
  most 10 Hz; slow-paced first boundaries, control replies and outcomes publish
  promptly. The client reduces every sequence, coalescing only paints. Serialized
  socket history is bounded to eight shared envelopes; idempotent replies have a
  16 MiB serialized budget (newest oversized reply retained).
- **11 Chromium tests passed**, including a new 10,000-node fixture with full
  scenario/service arrays and 10,000 historical jobs. It verifies bounded hidden
  DOM, JSON/job/connection pagination, search, topology, 50-update burst reduction
  without false resynchronization or resetting the current job/connection page,
  and phone overflow behavior. The final strengthened performance regression
  was rerun successfully. Svelte check: **0 errors,
  0 warnings**. `cargo build --release --locked` passed. Mechanical UI detector:
  `[]`. Desktop and 390px phone captures were visually reviewed; the resizable
  phone inspector now scrolls within its bounds rather than spilling past footer.
- Evidence stays ignored under `.impeccable/review/`: `optimization-*-results.json`,
  before/after replay files and `dashboard-large-*-after.png`; the checked-in
  performance regression lives in `ui/tests/performance.spec.ts`.

### Parallel Lua router VMs — 2026-11-12

- Enabled mlua's `send` support and moved VM execution onto bounded worker threads
  in fixed, sorted cohorts of eight. Shared effects use deterministic round-robin
  turns; read-only queries use shared world locks. Workers join before every
  world/task/enemy/message barrier. Lua globals, coroutines, module caches, fuel,
  memory caps and task durations retain their per-router semantics.
- `VANGUARD_SKIP_UI_BUILD=1 cargo test --quiet`: **48 tests passed**. An instruction
  hook rendezvous proves eight VMs execute Lua concurrently. Replay checks cover
  4, 5, 9 and 17 routers, fuel exhaustion, native task admissions, publications,
  pending handlers and runtime errors across cohort boundaries. Existing sandbox,
  allocator-limit, scoring, restart and teaching-agent checks pass.
- `cargo build --release --locked` passed. The 10,000-node final fixture's ordinary
  API traffic control still wins at **T45 / 999.91**, with zero Lua errors. Two
  runs match all **46 hashes**; final hash
  `44bd152cd7fd5bfc33744cbbde0f5f65cc9988e9b9d8bb24bcd7ecee8272014b`.
- Isolated Lua-only benchmark: 40 router VMs, busy-loop agent, 10 ticks, no hosts,
  attacks or scored services. Three-run median falls from **2.184 s sequential**
  to **0.383 s parallel** (about **5.7×** on this machine). This isolates VM speed,
  not whole-match throughput. Final-network traffic-control runs take 9.64–9.90 s;
  the shared world/enemy barrier and native effects remain serialized work.
- Snapshots/hashes record `parallel-native-round-robin-v1` and the fixed cohort
  size. Historical sequential replay hashes/task IDs need not match this scheduler.
  Benchmark scripts, saved old binary and replay evidence remain ignored under
  `.impeccable/review/`.

### Ten progressive networks — 2026-11-12

- Added `progressive/01-10-nodes/network.toml` through
  `progressive/10-10000-nodes/network.toml`, generated by `progressive/generate.py`.
  Total nodes: 10, 100, 250, 500, 1,000, 1,500, 2,500, 5,000, 7,500 and 10,000.
  LAN counts: 1, 2, 3, 5, 8, 12, 16, 24, 32 and 40. Initial infections:
  1, 2, 4, 8, 16, 30, 48, 96, 160 and 240. Initial sources per LAN increase
  from one to six. Each file publishes its own deadline.
- Every file passes independent checks for exact unique node counts, valid LAN
  ownership, connected/deduplicated router links, host/service references, unique
  weighted pairs, clients for every server, expected initial infections, and the
  500-node cap including routers and servers. Largest LAN at level 10: 451 nodes.
- All ten files pass the release engine's `--print-config` validation. Regenerating
  the suite produces byte-identical files. The progression adds dependency and
  infection complexity as well as scale; it does not claim a strict difficulty
  ordering for every possible policy.
- Normal-API traffic control full attempts: levels 01–05 win at T40, T72, T45,
  T45 and T89; level 06 loses at T278 (198.74 points). All six have zero runtime
  errors. Levels 07–10 were structurally/engine validated, not terminal-benchmarked.
  Detailed results live in `progressive/README.md`; replay evidence is ignored
  under `.impeccable/review/progressive/`.

### 10,000-node final fixture — 2026-11-12

- Added `final-test/network.toml`, reproducible with `final-test/generate.py`.
  Revised to exactly 10,000 unique nodes: 40 routers, 9,880 workstations,
  80 servers; one LAN/router, connected 54-edge topology, eight initial infections,
  29,640 unique valid required-service pairs, three pairs/client, every server used.
  Independently verified the 500-node/LAN cap including routers and servers:
  eight LANs each at 500, 351, 101 and 97 nodes; four each at 251 and 151 nodes.
- Four sites have mixed 497/348/248/148/98/94-workstation LANs and weighted local,
  within-site and remote service dependencies (71.43% cross-LAN service weight).
  Standard resource/infection budgets; 240-tick deadline; 14-tick recovery,
  90% required-service threshold and 30-tick total-outage grace.
- Release CLI accepted it. An ordinary-API, topology-independent
  evidence-led feasibility agent won at **T45 / 999.91** with zero runtime errors.
  Two full runs matched all **46 hashes**. Control source and replays remain in
  ignored `.impeccable/review/`, outside the distributable final-test files.
- Basic/Coordinated/Advanced passed 12-tick smoke runs with zero errors and eight
  remaining compromised hosts. These are not terminal loss benchmarks. Separately,
  the ordered-scan deadline bound is at most 400 HOST_SCANs/router by T240, even
  without other work; four campus sources are at local position 497.
- Regeneration byte-matches SHA256
  `2194c165026c48057240f99354173385ce538a6258c1c6ce0f65586729a95e9a`.
  The fixture is separate from the small default topology and public handbook
  sample/download collection.

### Stop immediately on verified full recovery — 2026-11-12

- Scored matches now enter FINISHED/VICTORY on the first completed recovery window,
  before the deadline. The core rejects further barriers and runtime step calls
  preserve the final state. Zero compromised alone is still insufficient.
- Remaining deadline ticks receive the verified state's actual service/clean
  fractions; no future simulation work executes. Partial-service coverage confirms
  this does not award perfect service or operational credit.
- **46 Rust tests passed**, including verification-window interruption, immediate
  victory, frozen state/score, restart reset, deterministic victory, and no victory
  for clean hosts without the required service. **10 dashboard browser tests passed**;
  UI Svelte check had zero errors/warnings. Release build passed.
- Real release/headless Tutorial 1 with Basic Defender and a 300-tick limit ends
  at **T40**, VICTORY, **988.70**. Two runs match all **41 hashes**. Real HTTP
  StepTicks(300) also stops at T40; further stepping is rejected; Restart resets
  tick and victory accounting. Replays: `.impeccable/review/early-victory/`.
- Updated scoring rules, mailbox lesson and match-rules dialog; rebuilt standalone
  VanMail. Documentation API/duration/link coverage passed. Older T300/T1000
  measurements below describe the previous deadline-continuation behaviour.

### Single-HTML distribution is the default build

- `bun --cwd vandocs run build` now emits only `out/VanMail.html` and clears stale
  generated multi-page output. All CSS, JavaScript, emails and file downloads are
  embedded. The generated distribution contains exactly one file (410,104 bytes).
- Preview serves this same file. An HTTP smoke check byte-compared both `/` and
  `/VanMail.html` with the distribution artifact successfully.
- All **5 documentation browser tests passed**, including offline navigation,
  new tabs and embedded downloads. The offline test asserts that `out/` contains
  only `VanMail.html`. Route-based regression fixtures use internal `.site-build/`.
- Documentation Svelte/TypeScript check: zero errors/warnings.

### Consistent explorer hierarchy and resizable panes

- Removed the Situation/LAN brief content; the default inspector shows Match score.
  Selected nodes alone have an X, which returns to the score panel.
- Topology/Table share overview and LAN scope. The overview table lists LANs and
  selecting a row shows only that LAN's nodes without changing Table mode.
- Added pointer/keyboard resize dividers for main columns, stacked panes, telemetry,
  debug perspectives and drawing height. Main widths/dock height persist locally;
  double-click resets and size bounds preserve useful content space.
- **10 dashboard browser tests passed**, covering LAN-table drill-down, scope/view
  retention, node close behaviour, pointer/keyboard resizing, reload persistence,
  dock collapse/reopen, debug resize, and tablet/phone overflow checks.
- UI Svelte check: zero errors/warnings. Release build with embedded UI passed.
  Updated and rebuilt the offline interface guide; handbook coverage passed.
- Inspected overview-LAN table, LAN-node table and resized 1440/390px captures.
  Mechanical detector returned `[]`.

### Self-contained links/downloads and reduced mailbox chrome

- Rebuilt `vandocs/out/VanMail.html` with native hash links for offline email
  navigation, including the five links in the 30-minute starter table and new tabs.
- Embedded a ZIP of all exercise agents, helper modules, networks, generator,
  default network and Network Agent.md. Individual attachments download exact source.
- All **5 mail Chromium tests passed**. The offline test follows every starter-table
  link, opens Tutorial 2 in a new tab, downloads/unzips the exercise archive and
  byte-compares key files with source, downloads Network Agent.md individually,
  and asserts zero HTTP requests with the context offline.
- Removed icon rail, global/list footers and promotional helper text. Inspected
  fresh 1440px/390px captures; both layouts fit. Svelte check has zero errors/warnings,
  coverage passes, production/standalone builds pass, detector returned `[]`.

### VanMail competition handbook, offline export, defenders and dbgguard

- Rust: **44 tests passed**. New checks execute all eleven new teaching/defender
  entrypoints on both tutorial networks, execute inline Lua in six new lessons,
  prove cross-LAN discovery plus remote-source blocking, and exercise opt-in,
  write-only namespacing, transactional debug rejection, deletion, clearing,
  view/storage limits and deterministic replay.
- Both Svelte checks: **zero errors and warnings**. Coverage verifies all 23 Lua
  functions, 19 durations, internal links, registered emails and actual attachment
  files across **43 emails**. A guard rejects build/install instructions in mail.
- Documentation build emits the multi-page site and **out/VanMail.html (~390 KiB)**.
  **5 Chromium tests passed**: all direct URLs, full API headings, generated index,
  navigation/search/read/star/new-tab behaviour, light/dark persistence, exact code
  copying, syntax tokens, desktop/phone layouts, and a disconnected `file://` run
  with zero HTTP requests. The resize check caught open desktop folders overflowing
  the phone toolbar; responsive folder state and compact phone actions fixed it.
- Dashboard: **9 Chromium tests passed** including the two-perspective debug
  fixture. The focused comparison test passed again after its sizing fixes:
  bounded pages, metadata, perspective selection and 1440/390px layout. Drawings
  scroll within 55dvh while the explorer remains usable below.
- `cargo build --release --locked` passed with the embedded current UI. HTTP smoke
  exercised the release binary with `--dbg --port 0`: health, embedded index,
  normal stepping, two real agents' drawings and Restart reset/flag persistence.
- A clean packaged-source temporary Git repository installed successfully with
  `cargo install --git file://... --locked --bin vanguard`. The installed binary
  reported version 0.1.0. Cargo's package whitelist excludes node_modules, docs
  bundles and build outputs. GitHub installation needs these changes published;
  no commit or push was made in the working repository.
- Seed 1, 300-tick generated star fixtures, one compromised workstation,
  default budgets, local required services, no debug instrumentation:

  | Agent | Total nodes | Outcome | Score | Runtime errors |
  | --- | ---: | --- | ---: | ---: |
  | Basic | 1,000 / 50 routers | VICTORY T300 | 999.91 | 0 |
  | Coordinated | 10,000 / 100 routers | VICTORY T300 | 999.97 | 0 |
  | Advanced | 10,000 / 100 routers | VICTORY T300 | 999.97 | 0 |
  | Ostrich | 1,000 / 50 routers | DEFEAT T127 | 214.52 | 0 |
  | Ostrich | Tutorial 1, 12 nodes | DEFEAT T57 | 63.45 | 0 |
  | Ostrich | Tutorial 2, 24 nodes | DEFEAT T57 | 75.81 | 0 |

  These prove the published fixtures, not arbitrary-size/mission guarantees.
  Replays and generated topologies are under `.impeccable/review/scale/`.
- Desktop/phone light and dark mail plus debug comparison captures were inspected
  in-thread. Mechanical detector returned `[]`. Finish notes:
  `.impeccable/review/vanmail-debug-finish.md`. `git diff --check` passed.

Earlier check results are retained below as history.

### Documentation mailbox / connection navigation / tick pacing

- Rust: **39 tests passed**, including minimum-duration pacing arithmetic,
  validation, published pacing state, and responsive Pause during a paced wait.
- Both `ui` and `vandocs`: Svelte/TypeScript **zero errors and warnings**.
  Cargo rebuilt and embedded the dashboard; SvelteKit exported the mail app to
  `vandocs/out/` successfully.
- Real-Rocket Chromium: **8 tests passed**. The two focused regression tests also
  passed after the responsive topology correction; the bounded large-snapshot
  test also passed again after that correction. Coverage includes sidebar
  table-to-map selection, a later-page host, connected remote peers despite a
  search filter, actual Cytoscape selected nodes/edges, readable connection rows,
  both keyboard shortcuts, clickable search badge, clean focus outline, and
  server-owned pacing after page reload.
- Static-mail Chromium: **3 tests passed**, covering all 20 direct lesson URLs,
  all 23 rendered API headings, folders, full-text search, persisted stars,
  read/unread, new-tab links, mobile search/reader/back navigation, accessible
  names for icon-only mobile actions, and overflow. All three passed again after
  the mobile accessible-name correction.
- Coverage: **23 API functions, 19 action durations and internal links** verified
  across 20 rewritten emails. Mechanical UI detector returned **no findings**.
- Desktop (1440px) and phone (390px) inbox/reader screenshots were inspected,
  along with updated explorer connection, search-focus and responsive captures
  under `.impeccable/review/`. Browser checks used cached Chromium via
  `PLAYWRIGHT_CHROMIUM_EXECUTABLE`.
- Independent finish review requested three fixes: phone graph scale, icon-only
  mail action names, and current product/design notes. The verdict pass scored
  all three **resolved / SHIP**, at that fix-list scope. Reports:
  `.impeccable/review/mail-explorer-finish.md` and `mail-explorer-verdict.md`.

Older check results below are retained as history.

### Current scoring / network explorer / web-only startup

- Rust: **37 tests passed** after removing the legacy terminal runner/tests.
  Six scoring tests cover perfect victory, routing-aware service, weighted pairs,
  outage/reset/deadline accounting, hidden offline infection, recovery revocation,
  validation, early runtime stop and fresh-run counters.
- Svelte check: **zero errors and warnings**. Production UI built/embedded through
  Cargo during browser tests. Documentation production export and TypeScript passed.
- Real-Rocket Chromium: **6 passed**, including controls, score DTOs, search/LAN
  navigation, server inventory, truth/evidence, telemetry, reconnect/resync,
  mobile source selection and bounded 2,048-host rendering. A 160-host LAN also
  verifies that a selected second-page host appears on the correct graph page.
- Static documentation Chromium: **2 passed**; coverage verifies **23 API calls**,
  **19 durations** and internal links across **20 pages**.
- Full scored defender replay repeated twice: all **1,001 hashes match**. Both
  finish at tick 1,000 with **VICTORY, 937.14 points** (562.50 service, 249.65
  security, 124.99 recovery). Final semantic hash:
  `4deec9492d3518328f61408decfd85869db3d773906b2e7b3f5f675fc097b9f5`.
  Ostrich reaches **DEFEAT at tick 71, 23.96 points**. These are the current
  repository network/seed results, not a competition-wide benchmark.
- Started the binary with `--port 0`, read its ASCII banner/assigned URL, verified
  `/api/health` at that actual URL and stopped cleanly via Ctrl-C/SIGINT.
  Ratatui/Crossterm references and dependencies are absent.
- Independent dashboard finish review: **SHIP** after verifying selection/page
  coherence and mobile telemetry selection. Screenshots: `command-workspace-1440`,
  `command-workspace-390`, `command-workspace-live`, `explorer-large`,
  `explorer-selected-page`, `explorer-mobile-dock` under `.impeccable/review/`.
- Formatting remains unavailable (`cargo fmt` is not installed).

Sections below retain prior verification history.

### Current single-agent startup and command workspace

- `cargo test --quiet`: **41 passed**, zero failures, including explicit TOML
  validation, required CLI inputs, VM isolation, replay determinism, restart and
  real-barrier StepTicks behavior with the inactive ostrich agent.
- `bun run check` in `ui`: **zero errors and warnings**.
- Real-Rocket Chromium integration: **4 passed**. Covers Start/Pause/Restart,
  single/multiple tick stepping, live logs/messages/topics/jobs/statistics,
  discovered observations, reconnect/resync/stale-run rejection, desktop/mobile
  layout and `/components` returning 404.
- `vandocs`: production static export succeeded; TypeScript check passed;
  coverage check verified **23 functions**, **19 durations**, and internal links
  across **18 pages**. Static-site Chromium integration: **2 passed**, covering
  rendered references, navigation and desktop/mobile layout.
- Reviewed `.impeccable/review/command-workspace-1440.png`,
  `command-workspace-390.png`, and `vandocs-1440.png`. Corrected mobile graph
  resizing and integrity-bar style collision, then reran dashboard checks.
- `cargo fmt --check` remains unavailable: the Cargo formatting command is not
  installed in this environment.

### Previous baseline (before startup/workspace changes)

- `cargo test`: **37 passed**, zero failures. Normal Cargo builds successfully
  run the locked Bun installation/static frontend build and embed the assets.
- `bun run check` in `ui`: **zero errors and warnings**.
- Browser integration against real Rocket:

  ```sh
  cd ui
  PLAYWRIGHT_CHROMIUM_EXECUTABLE=/home/mousebrilliant/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome bun run test:e2e
  ```

  **10 passed**. Covers live controls/pause/restart, reconnect and resync,
  stale-run/sequence-gap rejection, node selection, tabs/filters/job details,
  structured source-colored output and nested tables, direct messages, topic
  dialogs/Escape/focus return, desktop/mobile layouts, themes, and the preserved
  component gallery.
- Repeated headless execution:

  ```sh
  cargo run -- --headless --config config/default.json --ticks 100 --replay .impeccable/review/replay-b.json
  cargo run -- --headless --config config/default.json --ticks 100 --replay .impeccable/review/replay-c.json
  ```

  All **101 committed hashes matched**, including tick zero. Final hash:
  `691627d812a23507736f883594d844599df5be9c11da14efb5c9ff26cf02158f`.
  Tick 100 had **124 jobs, zero failed jobs, and zero runtime errors**. Separate
  runtime tests compare each barrier with different presentation pacing.
- Reviewed rendered desktop, narrow, dark/light and live-state screenshots in
  `.impeccable/review/dashboard-*.png`.
- `cargo fmt --check` was attempted but **could not run**: this environment has
  neither the Cargo formatting command nor `rustfmt` installed.

## Coverage notes and concrete limits

- Browser verification used cached Chromium. Firefox, WebKit and assistive
  technology have not been exercised.
- Native work has published deterministic size-based charges, not native CPU
  instruction counts. The supported standard library is intentionally restricted
  as documented in `decisions.md`.
- Protocol deltas replace changed top-level fields, including whole changing
  arrays. They are not per-element compressed diffs.
- Replay output verifies committed semantic hashes and provides a final snapshot;
  it is not a serialized Lua-heap checkpoint for mid-run restoration.
- The retained event window is bounded and explicitly reports eviction. Jobs and
  their terminal results remain in memory for the current run; no disk history
  database is provided.
- Tests cover the specified core timing, scope, filtering, VM isolation and
  synchronization scenarios; they are not an exhaustive Cartesian product of
  every action, graph, resource setting and browser interaction.
# Incremental engine and topology usability follow-up (2026-11-12)

- `VANGUARD_SKIP_UI_BUILD=1 cargo test --quiet`: 55 passing tests, including
  sparse/dense service invalidation and exact on-demand/shared result retrieval.
- `bun run check`: zero errors or warnings. Chromium suite: 13 passing tests;
  the topology regression was also rerun after the phone framing/readability fix.
  `cargo build --release --locked` passed. Mechanical UI detector: `[]`.
- Desktop and 390px phone captures reviewed: long host IDs remain separated,
  power/isolation appears below IDs, LAN headings remain inside group bounds,
  router running jobs open results, and discovered perspective defaults to owner.
  Tests also verify preserved camera state after changing session membership,
  globe button/wheel zoom and no document horizontal overflow.
- All 21 outbreak replay hashes and all 46 final-control hashes match `af44271`.
  Final control still finishes at T45 with score 999.91 and hash
  `44bd152cd7fd5bfc33744cbbde0f5f65cc9988e9b9d8bb24bcd7ecee8272014b`.
- Three-sample plain-headless level-10/T20 median: 1.160 → 1.074 seconds
  (about 7.4% lower wall time). Median maximum RSS: 199,364 → 97,760 KiB
  (about 51% lower). A separate replay sample: 3.618 → 2.953 seconds;
  do not interpret the single replay sample as a stable per-component speedup.
- Real final-network initial projection: 7,030,446 bytes full → 2,090,766 bytes
  compact (70.26% reduction). Live compact stepping, on-demand job retrieval and
  stale-run rejection passed. This measures initial snapshots, not socket deltas.
- Evidence: ignored `.impeccable/review/incremental-controlled-results.json`,
  `incremental-after-{replay,control}.json`, and Chromium `topology-labels-*.png`
  captures under `ui/test-results/`.
# Progressive containment defender and 100k extension (2026-11-13)

- Added self-contained `progressive/containment-defender.lua`, using only the
  normal Lua API and discovered local membership/traffic/health results. It
  quarantines suspects before diagnosis, prioritizes clean-host releases, limits
  concurrent restores to three, and patrols for same-LAN/history blind spots.
- Full terminal runs win all eleven published progressive fixtures. Levels 01–11
  finish at T40, T49, T42, T45, T59, T72, T56, T78, T87, T96, T99; corresponding
  scores are 977.96, 996.29, 996.31, 997.46, 998.31, 997.71, 998.49, 997.89,
  997.87, 997.89, 997.68. Every final state has 100% working required service,
  zero compromised/exposed hosts and zero summed per-agent runtime errors.
- Repeating level 10 matches all 97 per-tick replay hashes. Compact, source- and
  network-hashed evidence is checked in as `progressive/validation-results.json`;
  `progressive/verify.py` reproduces terminal verification and returns failure on
  non-victory, partial/incomplete coverage or aggregate agent errors.
- New level 11: 400 routers, 98,800 workstations and 800 servers = 100,000 total
  nodes, across four sites; 414 links, 2,400 initial infections, 296,400 required
  pairs, largest LAN 451 total nodes, seed 20272079 and deadline 720. Mission and
  resource defaults are unchanged. Independent count/uniqueness/connectivity/
  service-reference/LAN-cap checks and release `--print-config` passed.
- Generator reproduces level 11 byte-for-byte; all original ten fixtures remain
  byte-identical. Network SHA256:
  `53c32ab0840bf7721ab4ffafbb4b424a28e1d5b354e59f51ef90f64766b44d92`.
- The 100k replay run took about 266 seconds and peaked at 8,104,208 KiB simulator
  RSS on this machine, including full replay hashing/final snapshot output.
  This is not a plain-headless or frontend benchmark. Final state hash:
  `b85ad229a97db20ced5f28851ee05f44194b3bca69dd2539672babd09e724b24`.
- Raw evidence remains under ignored
  `.impeccable/review/containment-validation/`; no engine/scoring changes were used
  to obtain the victories.
