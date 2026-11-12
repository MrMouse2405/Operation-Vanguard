# Network explorer

Primary target: ui/src/routes/+page.svelte
Mode: Operate. Competitors inspect a live match, find risk, and diagnose their agent.

## Direction contract

THESIS: Search, filter, and drill into LANs instead of rendering a whole-world hairball.
OWN-WORLD: The supplied dashboard(1) HTML pins dark slate, emerald healthy,
amber exposure, red compromise, blue server inventory; compact sans controls and
monospace telemetry, consistent Lucide strokes.
STORY: Read match/service score, find a host, inspect actual truth or agent evidence,
then correlate jobs and messages without leaving the explorer.
FIRST VIEWPORT: Command header, clickable host/server totals, left hierarchy/search,
central aggregate health map or paginated table, right Match score/node inspector,
and central telemetry dock. LAN detail shows a bounded real graph.
SCOPE: Topology and Table share Network → All LANs → LAN. The overview table
lists LANs; opening a LAN row shows its nodes without changing the view mode.
PANES: Draggable, keyboard-operable dividers resize the main columns and telemetry,
and debug perspectives/drawing height. Narrow screens stack panes with horizontal
height handles. The node inspector alone has an X to return to Match score.
CURRENT GRAPH: Connections is the default detail view; there is no Physical/Sessions
switch. Render up to 80 matching page hosts plus up to 80 additional selected-node
connection endpoints, even beyond active filters/LAN, and their owning routers.
LAN groups imply attachment; arrows show live connections. The truth inspector
retains every inbound/outbound/transit record with linked endpoints, opened tick,
age and route. Sidebar nodes locate their graph scope/page. Below 500px graph
width, LAN groups stack; automatic Fit keeps zoom at least 0.85 and centers the
selected node, with pan for larger neighborhoods. The Ctrl/⌘ K badge is clickable
and search focus outlines the complete field. Seconds/tick sets minimum total
tick duration including computation during playback; 0 means full speed.
FORM: User-pinned explorer prototype; code-led; seed: none (user supplied).
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
