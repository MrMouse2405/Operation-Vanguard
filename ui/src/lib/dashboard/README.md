# Live dashboard

`/` is the map-first simulation dashboard. It uses dark slate panels, semantic
health colours and Lucide icons. The old `/components` showcase is removed.

## Runtime contract

WebSocket `/api/ws` sends:

```ts
{
  schema_version: 1,
  run_id: string,
  seq: number,
  tick: number,
  type: 'snapshot' | 'update' | 'ack' | 'error',
  request_id?: string,
  snapshot?: Snapshot,
  delta?: Partial<Snapshot>,
  base_seq?: number,
  error?: unknown
}
```

Snapshots and acknowledgements carry the complete committed projection. Updates
carry changed top-level fields in `delta` and the expected previous `base_seq`;
arrays replace atomically. Merge then normalize uses the same projection shape
as reconnect. Full-projection updates are also accepted. `seq` is a transport
sequence, independent of individual event sequence numbers. An update gap requests
`GetSnapshot`; a snapshot adopts its run; stale updates from retired runs are
discarded. Reconnect uses capped exponential backoff. Command buttons remain busy
until their matching request ID is acknowledged, errors, disconnects, or times out.

Commands are JSON objects, for both WebSocket and HTTP:

```ts
{ schema_version: 1, request_id: string,
  command: 'Start' | 'Pause' | 'Resume' | 'Restart' | 'GetSnapshot' |
    'StepTicks' | 'SetTickPacing',
  ticks?: number, seconds_per_tick?: number }
```

`GET /api/snapshot` may return a bare snapshot or a snapshot envelope. When the
browser has no WebSocket support, the dashboard polls this endpoint and sends
commands to `POST /api/command` with the same envelope; the POST response is an
ack/error envelope. HTTP fallback does not take over on ordinary disconnections.

The canonical DTO is typed in `types.ts`:

- Root: `schema_version`, `run_id`, `seq`, `tick`,
  `control: READY | RUNNING | PAUSED | ERROR | FINISHED`, `seconds_per_tick`.
- `scenario`: `router_count`, `seed`, `tick_ms`, `tick_limit`, `fuel_per_tick`,
  `instruction_quantum`, `memory_limit`, `max_payload_bytes`, `max_handlers`,
  `tick_cost` and any additional runtime-published configuration.
- `nodes[]`: numeric `id`, `type: ROUTER | WORKSTATION | SERVER`, numeric
  `router_id`, optional `label`, `power`, `compromised`, optional `exposure` and
  `shield: { policy, exceptions: number[], isolated }`. Flat `policy`,
  `exceptions`, `isolated` are accepted too. Routers never enter host infection
  totals.
- `links[]`: `{ source: number, target: number }`.
- `sessions[]`: `{ id, source: number, target: number, opened_at }`.
- `jobs[]`: `{ id, agent_id, router_id, source, action, target?, peer?, state,
  started_at, finished_at?, duration, fuel_spent, descriptor?, result?, failure? }`.
  Progress is elapsed committed ticks divided by actual admitted duration, not a
  timer. Terminal jobs remain inspectable.
- `agents[]`: `{ id, router_id, fuel, last_fuel, known: number[], observations: [],
  memory_bytes, peak_memory_bytes, coroutines, backlog, runtime_errors,
  allowance, slots?: { capacity, running, available } }`.
- `events[]`: `{ seq, tick, kind, agent_id?, source, data }`. `sequence` is accepted
  as an alias of event `seq`. Log data contains `values: unknown[]`; direct-message
  data contains `message_id`, `sender`, `recipient`, `payload`, `sent_at`, optional
  `delivered_at` and `outcome`. PubSub event data contains exact `topic` and its
  payload/metadata. Job events reference `task_id` or `job_id` for association.
- `topics[]`: `{ topic, subscribers: number[], publication_count, last_tick }`.
- `enemy_updates: number`, `history: { first_seq: number, evicted: number }`.

Missing observations are never reconstructed from world truth or job results.
The Discovered inspector displays only selected-agent recorded observations
matching a selected node: explicit targets, implicit own-router actions, SCAN
topology mentions, TRAFFIC_SCAN endpoints, or observed connection peers. It keeps
their original observed-at timestamps and clearly identifies stale knowledge.

`SetTickPacing` accepts a finite `seconds_per_tick` from 0 through 3600. It sets
the minimum continuous tick duration including computation/publication; only
remaining time is delayed, and over-budget ticks proceed immediately. Zero means
full speed. The owner thread remains responsive to controls while waiting, and
snapshot requests do not restart the clock. Pacing is presentation state outside
semantic hashes. It survives Restart; manual StepTicks executes barriers directly.

## Surface behavior

Desktop uses an explorer, map/table, inspector and collapsible telemetry dock.
At intermediate widths the inspector moves below the map; phones stack the
workspace. Each telemetry view retains its agent filter. Messages match sender
**or** recipient. The overview renders LAN aggregates, and LAN detail renders up
to 80 matching hosts plus up to 80 selected-node connection peers. Those extra
peers can be outside the search filter or LAN; their owning routers are included.
All connections remain listed in the inspector even if peers exceed that bound.

Sidebar node clicks switch to topology, open the right LAN, align pagination and
select the node. Graph arrows show selected-node live connections automatically,
including transit connections for selected routers. The inspector uses readable
direction, peer, opening tick and route rows with peer-navigation buttons.
Ctrl/Cmd K and its clickable platform-aware badge focus/select search. One focus
frame encloses the entire field and icon.

Event rendering is bounded to the latest 100 matching events, expandable to 500,
with separate runtime-evicted and display-hidden counts. JSON trees use native
details, preserve expansion through keyed tick updates, render values as text,
and stop recursion at depth 20. Source colors are stable hashes of normalized
file/source identifiers. Topic and job detail dialogs use the native modal focus
trap, Escape dismissal and focus return.

## Verification

```sh
bun run check
bun run build
bun run test:e2e tests/dashboard.spec.ts
```

The Playwright config manages a real Rocket process on port 18124. Set
`PLAYWRIGHT_CHROMIUM_EXECUTABLE` to an installed Chromium if the Playwright-managed
browser revision is unavailable. Tests exercise runtime controls, paused ticks,
node selection and observation scope, each agent-filtered view, job details and
focus return, restart identity, responsive overflow and light/dark screenshots.
Screenshots are written to `.impeccable/review/` at repository root.
