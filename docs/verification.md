# Verification — 2026-11-11

## Executed checks

### 10,000-node final fixture — 2026-11-12

- Added `final-test/network.toml`, reproducible with `final-test/generate.py`.
  Independently checked exactly 10,000 unique nodes: 24 routers, 9,928 workstations,
  48 servers; one LAN/router, connected 34-edge topology, eight initial infections,
  29,784 unique valid required-service pairs, three pairs/client, every server used.
- Four sites have mixed 1,200/400/160–161-workstation LANs and weighted local,
  within-site and remote service dependencies (71.43% cross-LAN service weight).
  Standard resource/infection budgets; 240-tick deadline; 14-tick recovery,
  90% required-service threshold and 30-tick total-outage grace.
- Release CLI `--print-config` accepted it. An ordinary-API, topology-independent
  evidence-led feasibility agent won at **T45 / 999.91** with zero runtime errors.
  Two full runs matched all **46 hashes**. Control source and replays remain in
  ignored `.impeccable/review/`, outside the distributable final-test files.
- Basic/Coordinated/Advanced passed 12-tick smoke runs with zero errors and eight
  remaining compromised hosts. These are not terminal loss benchmarks. Separately,
  the ordered-scan deadline bound is at most 400 HOST_SCANs/router by T240, even
  without other work; four campus sources are at local position 1,200.
- Regeneration byte-matches SHA256
  `5eb49c133f328f39299ef6312f9ba25267cbb73cee195c4cd989c85455fd0938`.
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
