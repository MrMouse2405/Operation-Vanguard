The telemetry dock starts collapsed below the explorer. Click a tab to open it. The chevron collapses it again; the agent selector changes whose work you are looking at.

## Which tab answers which question?

| Tab | Look here for |
| --- | --- |
| Activity | Engine events: task completion, infection, resets and runtime errors |
| Jobs | Running/completed work, task progress and timing |
| Output | Your `print` calls, agent, file, and expandable values |
| Messages | Direct send/delivery records and payloads |
| Pub / Sub | Topics, subscriptions, publications and intended recipients |
| Tasks | Agent task records and results |
| Statistics | Fuel, memory, activity and runtime diagnostics |

**No output** can mean your agent has not printed yet, the wrong agent is selected, or its code failed before the print. Look at Activity before adding a print inside every loop. Printing also costs fuel.

## Scores are more useful than a victory screenshot

The **Match score** panel shows service, security and recovery points, outcome, service availability and recovery/outage counters. Recovery points during play are provisional. Isolating everything may turn the map quiet while your service score collapses. Thirty consecutive zero-service ticks means default defeat.

History is bounded in the dashboard. The footer reports evicted events. Use a fresh run or a headless replay when you need a reproducible record, rather than assuming the browser retained the whole day.

## Compare your agent’s drawings

Launch with `--dbg`. A **Compare agent drawings · dbgguard** button appears above the summary. Open it and choose a left and right perspective: two views from one router, or views from different routers. Each pane shows update tick, node/edge counts, a paginated drawing, view notes and expandable metadata.

These are **agent-authored claims**, not truth. The regular explorer remains the truth/evidence reference. Drawings show up to 80 nodes per page; an edge is drawn only when both endpoints are on that page. [The reconstruction exercise](/docs/debug/reconstruction/) creates two views for you to compare.

If no perspectives are listed, run a script that calls `dbgguard.view`. Enabling the flag does not automatically scan the network or invent a map for your agent.
