# Vanguard command workspace

Static SvelteKit + Cytoscape frontend served by Rocket. The layout follows
`operation_vanguard_dashboard(1).html`: global search, LAN hierarchy, aggregate
health map, paginated node inventory, server shortcuts, score/situation brief,
node inspector and collapsible live telemetry dock. Icons use Lucide.

```sh
# From the repository root
cargo run -- --script vanguard/agents/defender.lua --network network.toml
bun --cwd ui run dev
bun --cwd ui run check
bun --cwd ui run build
bun --cwd ui run test:e2e
```

The Vite dev server proxies `/api` to Rocket on port 8000. No simulation or mock
data lives in the frontend. StepTicks processes real backend barriers. Playback
speed, graph updates and filters cannot alter simulation results.

`src/lib/dashboard/connection.svelte.ts` handles the versioned protocol and reducer.
`src/lib/dashboard/Topology.svelte` mounts Cytoscape only in the browser, batches
updates and preserves viewport/selection. Session and physical layers are distinct.
`NodeInspector.svelte` keeps agent observations separate from truth.

The former `/components` page has been removed. Existing reusable UI components
remain in `src/lib/components/ui`; they are not a public gallery route.

API documentation is the independent static Fumadocs site in `../vandocs`.
