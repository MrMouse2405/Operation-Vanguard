# VanMail

The Operation Vanguard handbook as an Outlook-style inbox. Built with SvelteKit,
adapter-static, Tailwind, shadcn-svelte components and Lucide icons. No simulation
backend or JavaScript server is needed in production.

Forty-three character-led emails teach first-year CS students the rules, scoring, network
files, strategy, tasks, messaging, sandbox and complete Lua API. The default
learning path orders them from getting started through the reference material.
Senders include Capt. Network, Sgt. Scanner, and Lt. Task.

```sh
bun install --frozen-lockfile
bun run dev             # http://localhost:3001
bun run check           # Svelte + TypeScript diagnostics
bun run check:coverage  # 23 functions, 19 durations, internal links
bun run build           # One distributable: out/VanMail.html
bun run preview         # Preview the production build on port 3001
bun run test:e2e        # Build + Chromium integration on port 18125
```

Stop a previous docs dev process before starting the new SvelteKit dev server.
If the required Playwright browser revision isn't installed, set
`PLAYWRIGHT_CHROMIUM_EXECUTABLE` to an installed Chromium executable.

## Mailbox behaviour

- Learning path, All mail, Unread, Starred, and thirteen topic folders.
- Briefing directory, seven-hour rules, 30-minute path, two tutorials, Interface
  guide, Lua guide, example agents, Network Samples and optional dbgguard workshop.
- Light/dark themes via ModeWatcher; syntax highlighting and copy on every fence.
- Full-text search across all folders, including exact Lua names. Ctrl/Cmd K or
  the shortcut badge focuses search; Escape clears it.
- Email selection updates the current file's `#/docs/.../` hash. “Open in new tab”
  opens the same HTML file at that email; Copy email link copies its full address.
- Read state and stars persist in browser local storage. Storage failures do not
  prevent reading. No account, real mail service, or external data is involved.
- Desktop: folder pane, message list, reading pane. Phone: inbox and email views,
  with an Inbox back link and collapsible folders. Search reveals matching mail
  even while reading an email. Arrow keys move between focused message links.

## Editing lessons

Edit plain Markdown in `content/mail/*.md`. `src/lib/mail.ts` owns the order,
topic, sender, subject, preview and canonical URL for each email. Markdown is
authored project content, compiled by marked; search input is never rendered as
HTML. `src/api.lua` is the API acceptance contract, and the Rust action registry
supplies effective costs. Keep examples and duration tables consistent with both.

The component sources live in `src/lib/components/ui/`; `components.json` contains
the shadcn-svelte configuration. `src/routes/mail.css` owns the Outlook palette,
responsive layout, prose and code treatment. No external fonts, analytics, search
endpoint, Fumadocs or Next.js runtime is used. Share `out/VanMail.html` directly,
or upload that one file to any static host.

## Single-file distribution

`bun run build` bundles the complete handbook into **out/VanMail.html**, the only
file in the distribution directory. It clears previous generated multi-page output.
Share that file alone. It works directly from a file URL with no fetches, external
assets or server; routing uses hashes. The offline entry reuses the same Mailbox,
content renderer, themes and copy controls as the site. Browser clipboard/local
storage permissions still apply. Competitor mail contains no installation/build
instructions; maintainer instructions live in `docs/install-from-git.md`.

All offline email links are native hash URLs, including the starter table,
directory, previous/next actions and new-tab links. **Download example files**
exports an embedded ZIP containing all agents, helper modules, network samples,
the topology generator, `Network Agent.md` and the default `network.toml`, preserving
the paths used in the lessons. Attachment blocks also offer individual downloads.
The standalone file never needs a repository checkout or a network fetch for these.
The mailbox has no icon rail, global footer, promotional folder note or list footer.

`<!-- attachment:agents/examples/solo.lua -->` includes the real runnable file
as a highlighted, copyable block. Network attachments use the same mechanism.
The briefing directory is generated from the manifest, ensuring every other mail
appears under its category.

The browser regression suite also builds route-based fixtures into `.site-build/`
to check the shared mailbox component. These are internal test artifacts; the
distribution remains the single HTML file in `out/`.
