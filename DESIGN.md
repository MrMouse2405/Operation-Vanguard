---
name: Operation Vanguard
description: A slate network command workspace and a blue-and-white documentation mailbox.
colors:
  canvas: "#0a1016"
  panel: "#101922"
  line: "#253440"
  muted: "#93a5b5"
  text: "#e0e9ef"
  emerald: "#54dfb3"
  amber: "#f8c176"
  red: "#ff7782"
  blue: "#90b8fc"
  control: "#19232d"
  control-border: "#354350"
  control-hover: "#263540"
  primary-ink: "#082219"
  primary-hover: "#83efca"
  selected: "#20333e"
  aggregate: "#12232e"
  aggregate-border: "#38515f"
  mail-background: "#ffffff"
  mail-text: "#242424"
  mail-primary: "#0067b8"
  mail-app-bar: "#0f6cbd"
  mail-accent: "#e6f1fb"
  mail-accent-ink: "#005a9e"
  mail-canvas: "#f5f5f5"
  mail-muted: "#616161"
  mail-line: "#e0e0e0"
  mail-input: "#b8b8b8"
  mail-search: "#eef6fc"
typography:
  body:
    fontFamily: "system-ui, sans-serif"
    fontSize: "13px"
  title:
    fontFamily: "system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 550
    lineHeight: 1.3
    letterSpacing: "-0.4px"
  telemetry:
    fontFamily: "monospace"
    fontSize: "10px"
  section-label:
    fontFamily: "monospace"
    fontSize: "10px"
    fontWeight: 700
    letterSpacing: "1.5px"
  mail-body:
    fontFamily: "'Segoe UI', system-ui, sans-serif"
    fontSize: "15px"
    lineHeight: 1.75
  mail-subject:
    fontFamily: "'Segoe UI', system-ui, sans-serif"
    fontSize: "25px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.4px"
  mail-list-subject:
    fontFamily: "'Segoe UI', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 500
    lineHeight: 1.5
rounded:
  control: "3px"
  surface: "4px"
  mail-control: "6px"
spacing:
  compact: "7px"
  small: "12px"
  medium: "18px"
  large: "20px"
  mail-prose: "18px"
  mail-code: "16px"
components:
  button-primary:
    backgroundColor: "{colors.emerald}"
    textColor: "{colors.primary-ink}"
    rounded: "{rounded.control}"
    padding: "7px 11px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-secondary:
    backgroundColor: "{colors.control}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "7px 11px"
  lan-card:
    backgroundColor: "{colors.aggregate}"
    textColor: "{colors.text}"
    rounded: "{rounded.surface}"
    padding: "14px"
  mail-button-outline:
    backgroundColor: "{colors.mail-background}"
    textColor: "{colors.mail-text}"
    rounded: "{rounded.mail-control}"
    padding: "0 12px"
    height: "32px"
  mail-button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.mail-text}"
    rounded: "{rounded.mail-control}"
    padding: "0 12px"
    height: "32px"
  mail-message-selected:
    backgroundColor: "{colors.mail-accent}"
    textColor: "{colors.mail-text}"
    padding: "16px 32px 15px 20px"
  mail-search:
    backgroundColor: "{colors.mail-search}"
    textColor: "{colors.mail-text}"
    rounded: "{rounded.surface}"
    padding: "0 10px"
    height: "34px"
---

# Design System: Operation Vanguard

## Overview

The user-supplied `operation_vanguard_dashboard(1).html` pins the visual direction: dark slate, semantic emerald/amber/red/blue, compact sans-serif controls, monospace telemetry, and consistent Lucide strokes. The shipped explorer supports searching, filtering, and drilling into LANs while preserving the engine as the source of truth.

The user-approved documentation replacement is a distinct Outlook-style mailbox: blue application chrome, pale gray folders, white message surfaces, fine separators and roomy reading typography. This is a multi-surface system: the mailbox extends the product without replacing the dashboard identity.

**Key Characteristics:**
- Dense, bordered operational panels with restrained depth.
- Aggregate overview, paginated inventory, bounded LAN detail.
- Explicit separation of operator truth, agent evidence, and match scores.
- A blue-and-white documentation mailbox with readable lessons and familiar mail states.

Source of truth: `ui/src/lib/dashboard/dashboard.css`, `explorer-fixes.css`, `Explorer.svelte`, `NodeInspector.svelte`, `Topology.svelte`, and `ui/src/routes/+page.svelte`; for mail, `vandocs/src/routes/mail.css`, `vandocs/src/lib/Mailbox.svelte`, `mail.ts`, and the local shadcn-svelte button/input/separator components. Dashboard frontmatter records the active explorer overrides rather than the older dashboard base palette. The removed component gallery is not the dashboard reference. Surface contracts live in `.impeccable/surfaces/network-explorer.md` and `docs-mail.md`.

## Colors

Slate canvas and panel layers support light text and muted metadata. Emerald marks healthy state, primary execution controls, focus, and active underlines; amber marks exposure; red marks compromise; blue identifies server inventory. Labels and counts carry meaning alongside color.

**The State Before Decoration Rule.** Use semantic accents for real state, selection, and actions; keep structural surfaces slate. Graph rendering retains its implemented base dashboard state colors, rather than silently implying every renderer uses the explorer override values.

Mail uses its own primary blue for actions and focus, a blue application bar, pale blue selection, white message surfaces, gray canvas/folders and muted gray metadata. Mail blue does not indicate dashboard server state. Unread messages combine heavier subject/sender text with a blue marker; stars combine an icon state with a warm highlight.

## Typography

System sans-serif handles identity, headings, controls, and explanatory copy. Monospace handles IDs, ticks, compact section labels, table values, evidence, and telemetry. Tabular numerals stabilize live counts. Situation headings use the title role; summary counts are larger (25px), while metadata is deliberately compact (8–11px). Preserve readable handbook typography rather than importing dashboard density into prose.

Mail uses Segoe UI with system fallbacks: base UI (14px), message subjects (13px), reading subject (25px), section headings (19px) and prose (15px / 1.75) limited to (72ch). Unread subjects and senders use weight (650). Code uses ui-monospace / Cascadia Code, with block text (12px / 1.65). At phone widths, prose remains (15px), list subjects become (14px) and reading subjects (24px).

## Layout

Desktop composition: command header, horizontally scrollable summary filters, then left explorer / flexible center / right inspector. The standard columns are (258px / minmax(0,1fr) / 300px). Header minimum height is (65px); the summary strip is (72px). The center owns breadcrumbs, view switch, scope, visual inventory, pagination context, and the collapsible telemetry dock.

At widths up to (1250px), side columns narrow and an expanded dock wraps to retain its agent selector. Up to (1000px), the inspector moves below the explorer and center. Up to (650px), the workspace stacks, aggregate cards form two columns, and command controls wrap onto a full-width row. The mobile inspector can expand with its content; summary filters and telemetry tabs scroll horizontally. Keep the agent selector available when mobile telemetry is expanded.

The standalone `vandocs/` Read surface is VanMail: a (56px) blue app bar above a (56px) app rail, (224px) folder pane, (330px) message list and flexible reading pane. The toolbar is at least (48px); email sheets reach (1000px) maximum width. List and reading regions scroll independently. At (1250px) and (1000px), columns and sheet padding contract. Folders start collapsed at widths up to (800px) and follow that breakpoint when resizing. At (650px), the app rail disappears, the app bar wraps search onto a full row, and inbox and reading become separate views with an Inbox return link; folders can be toggled. Code and reference tables scroll horizontally. Reduced-motion preferences remove smooth scrolling/transitions; print isolates the email.

The top-right account block, icon rail, global footer, list footer and promotional helper copy are removed. The folder pane starts at the left edge; the message and reading panes use the recovered space. A labelled moon/sun button toggles ModeWatcher light/dark preference. Dark mail uses ink (#e3e8ee), surfaces (#20262d), canvas (#171b20), separators (#3b4652) and link/focus blue (#8bc7ff); the application bar keeps its recognizable blue. Code fences have a language header, persistent Copy code action and token colours for Lua, TOML, shell, Python and JSON. The toolbar provides an embedded example ZIP and attachment blocks provide individual downloads. The same component ships in the website and a self-contained offline HTML bundle, where internal links use native hashes.

The optional dbgguard comparison expands above the host summary. Two labelled perspective selectors, update ticks, paginated drawings, view notes and expandable node/edge metadata use the existing slate system. Each graph renders 80 nodes per page; phone layouts stack panes and use four graph columns. The comparison scrolls within 55dvh; the rest of the dashboard retains an operational-height explorer below it. This view labels all content as agent-authored claims and never implies invented metadata is engine truth.

Connections detail groups hosts by LAN. Below a graph-container width of (500px), LAN groups stack vertically and use fewer columns with larger node labels. Automatic Fit clamps its narrow-screen zoom to at least (0.85) and centers the selected node (or first node), keeping larger neighborhoods pannable rather than shrinking everything to fit. Manual zoom remains available.

## Elevation & Depth

The workspace is flat and tonal: one-pixel rules, darker inset search fields, selected row fills, and thin active underlines establish hierarchy. Aggregate cards use a slightly lifted slate tone, not floating marketing-card shadows. Dialogs alone use the substantial overlay shadow recorded in the sidecar. Hover changes fill and border; keyboard focus uses an emerald outline (2px) with offset (2px).

Mail is also flat: fine gray separators, white sheets against gray reading space and pale blue row fills establish depth. The selected message has a thin inset blue rule. Mail focus uses a blue outline (2px) inset by (2px); the app-bar search instead outlines the whole search container in white with (2px) offset.

## Shapes

Controls have restrained corners (3px); search fields and aggregate cards use (4px). Tables, tabs, summary filters, and major panel boundaries remain rectangular. Lucide icons use consistent outline strokes and compact sizing (typically 13–17px, larger for identity and selected-node icons). Cytoscape uses its own outlined host/server glyphs and diamond routers. Do not revive the retired gallery's universal diagonal-cut surfaces.

Mail uses rectangular panes, gently rounded search/folder/email surfaces (4px), shadcn-svelte controls (6px) and circular sender/account avatars. The phone email sheet loses its border and radius. Lucide outline icons support mail actions and folders.

## Components

- **Command header:** shield identity, actual tick and runtime state, Start/Pause/Resume, single-tick and bounded forward-step controls, restart, and scenario information. Seconds/tick accepts (0–3600) and sets minimum total tick duration including computation during continuous playback; (0) is full speed. Pending/invalid runtime states govern controls; disabled controls retain their footprint with reduced opacity.
- **Summary filters:** total, compromised, exposed, healthy, servers, and infected servers are clickable inventory filters. Active filters use an emerald bottom line. Counts exclude routers; server counts remain a distinct inventory category.
- **Left explorer:** global host search, state/type filters, server shortcuts, hierarchy/results switch, and paginated lists. Search accepts IDs, hostnames, LAN/router and agent identifiers. The clickable Ctrl/⌘ K badge and shortcut focus/select search; focus outlines the entire container, not the inner input. Sidebar node selection locates its graph scope/page and clears excluding filters when needed. Escape backs out of selection and scope when no dialog is open.
- **Aggregate cards:** overview tiles summarize LAN host health and server counts; health strips represent proportions. They are explicitly aggregates, not physical routing links. Overview/hierarchy pages contain up to (48) LANs; node-result pages contain (40) hosts.
- **Inventory and Connections graph:** the table pages (80) hosts at a time. Connections is the default detail graph, with no Physical/Sessions switch. Cytoscape renders up to (80) matching page hosts plus up to (80) additional selected-node connection endpoints and their owning routers, including peers outside current filters/LAN. LAN fabric implies attachment; directed dashed arrows represent live connections with both endpoints rendered. Selecting a host in map mode or returning from the table locates its page. Preserve zoom, pan, fit, labels, state legend, omitted-peer hint and the table navigation path.
- **Inspector:** the default panel contains Match score; the Situation/LAN brief, introductory helper copy, server/priority lists and generic close button are removed. Selecting a node exposes True state versus Discovered evidence and a functional X that returns to Match score. Operator-only truth must remain explicit. Agent evidence retains perspective, observed tick/age, unknown fields, undiscovered states, and stale-scan context.
- **Shared view hierarchy:** Topology and Table both follow Network → All LANs → selected LAN. The overview table lists up to 48 LANs; clicking a row drills into up to 80 nodes per page while retaining Table mode. View switches preserve the current scope.
- **Resizable panes:** seven-pixel dividers resize explorer/center/inspector columns, telemetry height, debug perspectives and drawing height. Separators support pointer capture, arrow keys, Shift steps, Home/End limits and double-click reset. Main widths and dock height persist locally; viewport constraints preserve usable minimums. Stacked phone/tablet layouts use height dividers.
- **Connection rows:** the truth inspector lists every relevant inbound, outbound or transit connection, independently of the graph bound, with ID, linked peer/endpoints, opened tick, age and route or Same LAN. Rows use (12px) text, generous separation and wrapping route metadata.
- **Scores:** show engine-provided total (1,000), Service (600), Security (250), and provisional Recovery (150), with outcome, working service, recovery window, and outage counters. Unscored scenarios say practice; missing data stays connecting or unknown.
- **Telemetry dock:** collapsed by default; Activity, Jobs, Output, Messages, Pub / Sub, Tasks, and Statistics expand within the center. LAN scope selects the owning Vanguard; the agent selector permits another perspective. Preserve scrollable tabs, collapse/expand state, actual events, and explicit empty states.
- **Mail primitives:** local shadcn-svelte outline/ghost buttons use small (32px) and icon (36px) sizes; separators use one-pixel gray rules. Search is a pale field with a clickable platform-aware Ctrl/⌘ K badge; its focus treatment belongs to the whole field.
- **Mailbox navigation and reading:** ordered sender/subject/preview links, thirteen topic folders, Learning path, Unread and Starred views; all-mail search matches sender, subject and body regardless of folder. Read/star preferences persist locally when available. Stable lesson URLs, Open in new tab, Copy email link and previous/next links support sharing and sequence. Character-led emails use distinct greetings, sign-offs, plain first-year-CS explanations and tested runnable attachments; the shipped sequence contains (43) lessons, (23) Lua functions and (19) action durations. The first briefing indexes every other email by category.

## Do's and Don'ts

### Do:
- **Do** preserve the pinned slate palette, semantic state labels, and Lucide outline consistency.
- **Do** keep aggregate health, physical LAN detail, operator truth, and recorded evidence explicitly distinguished.
- **Do** retain pagination, selected-host page alignment, and the expanded mobile telemetry selector.
- **Do** preserve the mailbox's blue-and-white identity, whole-field search focus and spacious reading typography alongside the slate dashboard.
- **Do** describe verification precisely: the supplied completion evidence reports 39 Rust, 8 dashboard browser and 3 docs browser tests passing, both Svelte checks with zero errors/warnings, and a mechanical UI detector result of `[]`. Mail and explorer captures cover 390px and 1440px; the latest mobile graph layout is source-verified, with screenshot recapture pending.

### Don't:
- **Don't** substitute decorative counters, invented observations, or inferred success for engine data.
- **Don't** turn the explorer into a whole-world graph or restore the retired gallery's visual rules.
- **Don't** treat the reported Chromium checks as proof of comprehensive accessibility or cross-browser verification.
