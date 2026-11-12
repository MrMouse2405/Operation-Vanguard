Start with the LAN overview, not an attempt to fit ten thousand machines on a postcard.

## Find something

- Use the dashboard search, or **Ctrl/Cmd K**, for a node ID, LAN, router or agent.
- The left hierarchy groups hosts by LAN. Search results and inventory are paginated.
- Click a LAN to narrow scope. The top summary buttons filter compromised, exposed, healthy or server hosts.
- Choose **Topology** or **Table**. Both follow the same scope: **Network → All LANs → a selected LAN**. At All LANs, Table lists LANs with host/server and risk counts. Click a LAN row to see its nodes, staying in Table. Topology shows LAN aggregates at the overview and the bounded connection graph inside a LAN.
- Use page controls for more hosts, **Fit** to frame the drawing, and pan/zoom to inspect it. Selecting a host locates its page.

## Arrange your workspace

Drag the dividers between the explorer, main view and Match score/node inspector to resize them. The center takes the remaining space. Open a telemetry tab, then drag its top divider to adjust its height. Debug comparisons have dividers between perspectives and beneath the drawings. On narrow screens, panes stack and the handles resize their heights.

You can also focus a divider with Tab and use arrow keys (Shift for larger steps), Home/End for its limits, or double-click it to reset. Main column widths and telemetry height are remembered in this browser.

Selecting a node opens its inspector. Its **X** closes that node and returns the panel to **Match score**. The score view has no close button.

## Read the lines correctly

LAN grouping represents attachment. Directed connection arrows represent current sessions. Same-LAN traffic does not pass a router filter. Cross-LAN traffic follows the shortest physical route; the simulator does not route around a blocked path just because you drew another cable.

A missing connection arrow is not proof of an unreachable service. Required-service checks are read-only probes; they do not create visible sessions. The inspector lists relevant connections even when graph limits omit an endpoint.

## True state versus Discovered evidence

**True state** is the operator’s view: actual power, compromise, isolation, rules and exposure. **Discovered evidence** is what a particular agent learned. Check the perspective, scan type and **observed tick**. Old evidence is old evidence.

Example: True state says COMPROMISED at tick 20. Agent evidence says CLEAN, observed at tick 5. The agent has not secretly lied. The world changed after its scan. Your code needs a new observation.

## What the colours mean

Green: healthy. Amber: active exposure. Red: established compromise. Blue identifies server inventory. Read the accompanying labels too; router immunity is not the same as a clean host.

Escape backs out of a selection/scope when no dialog is open. If you lose a host behind a filter, search its ID again. It is usually less embarrassing than announcing that the server has escaped.
