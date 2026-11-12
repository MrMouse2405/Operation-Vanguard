import { marked, Renderer } from 'marked';
import hljs from 'highlight.js/lib/core';
import lua from 'highlight.js/lib/languages/lua';
import ini from 'highlight.js/lib/languages/ini';
import bash from 'highlight.js/lib/languages/bash';
import python from 'highlight.js/lib/languages/python';
import json from 'highlight.js/lib/languages/json';
hljs.registerLanguage('lua', lua);
hljs.registerLanguage('toml', ini);
hljs.registerLanguage('sh', bash);
hljs.registerLanguage('python', python);
hljs.registerLanguage('json', json);

export const folders = ['Briefing', 'Tutorial', 'Interface guide', 'Lua guide', 'Getting started', 'Network basics', 'Taking action', 'Teamwork', 'Example agents', 'Network Samples', 'Debug workshop', 'Lua API', 'Quick reference'] as const;
export type Folder = typeof folders[number];
const voices: Record<string, { greeting: string; signoff: string }> = {
  'Maj. Inbox': { greeting: 'Right. Everyone stop forwarding things.', signoff: 'Seven hours. Keep it useful,' },
  'IncreaseShareholderValue Corporation': { greeting: 'Dear valued future procurement success,', signoff: 'Synergistically yours, Customer Acquisition Division,' },
  'OCdt Syed': { greeting: 'I tried it. Here’s what happened.', signoff: 'Ask before the deadline, preferably,' },
  'OCdt Gerami': { greeting: 'Small example first.', signoff: 'Run it, read it, change one thing,' },
  'Capt. Network': { greeting: 'Pull up a chair.', signoff: 'Get a run going, then tell me what happened,' },
  'Maj. Packet': { greeting: 'Map attached.', signoff: 'Watch the local links,' },
  'Col. Scoreboard': { greeting: 'For your records:', signoff: 'I’ll be reading the service column,' },
  'Lt. Clock': { greeting: 'A timing correction:', signoff: 'Precisely,' },
  'Sgt. Sandbox': { greeting: 'Before you ask:', signoff: 'Within the stated limits,' },
  'Lt. Task': { greeting: 'Requests go here.', signoff: 'Keep your task IDs,' },
  'Sgt. Scanner': { greeting: 'What did you observe?', signoff: 'Put a tick on it,' },
  'Maj. Firewall': { greeting: 'About those open connections:', signoff: 'Check both endpoints,' },
  'Capt. Restore': { greeting: 'Morning. How’s the patient?', signoff: 'Send it back working,' },
  'Col. Debug': { greeting: 'Show me the run.', signoff: 'Keep the failing seed,' },
  'Lt. Relay': { greeting: 'Got your note.', signoff: 'Receiver’s on,' },
  'Capt. Runtime': { greeting: 'Reference copy:', signoff: 'Filed under v1.3,' },
  'Capt. Identity': { greeting: 'Before we process this request:', signoff: 'Use the correct register,' },
  'Sgt. Ready': { greeting: 'One last check:', signoff: 'Ask which kind of finished,' },
  'Sgt. Constants': { greeting: 'As requested.', signoff: 'No amendments,' }
};
const lessons: [string, Folder, string, string, string][] = [
  ['briefing', 'Briefing', 'Maj. Inbox', 'READ FIRST: seven hours and far too many forwarded emails', 'The CO wants an evaluation. Here is the clean list, the rules, and where to ask for help.'],
  ['vendor', 'Briefing', 'IncreaseShareholderValue Corporation', 'Vanguard™: your routers, now with additional capabilities', 'Sensing. Acting. Shielding. Command. Procurement would like a word.'],
  ['operation', 'Briefing', 'OCdt Syed', 'Operation Vanguard: the simulator behind the sales pitch', 'What fuel actually measures, what ticks do, and what we are competing to achieve.'],
  ['tutorial/one', 'Tutorial', 'OCdt Gerami', 'Tutorial 1: one router, ten workstations, one server', 'Your first 15 minutes: launch, inspect, scan, and try a local patrol.'],
  ['tutorial/two', 'Tutorial', 'OCdt Syed', 'Tutorial 2: two routers and a reason to send messages', 'Twenty workstations, two servers, and two private notebooks.'],
  ['interface/start', 'Interface guide', 'Capt. Network', 'Which button actually starts the thing?', 'Start, pause, stepping, pacing, restart, and getting a clean comparison.'],
  ['interface/explorer', 'Interface guide', 'Maj. Packet', 'Find the machine before you blame it', 'Search, LANs, inventory, connections, and true state versus evidence.'],
  ['interface/telemetry', 'Interface guide', 'Col. Debug', 'The useful part is probably in Output', 'Jobs, logs, messages, fuel, scores and agent drawings.'],
  ['lua/basics', 'Lua guide', 'OCdt Gerami', 'Lua: enough syntax to stop fighting the punctuation', 'Variables, booleans, if, loops, comments and the surprisingly important ~=.'],
  ['lua/tables', 'Lua guide', 'Capt. Identity', 'Tables are lists. Also maps. Label the drawer.', 'Indexing from one, ipairs versus pairs, copies and missing values.'],
  ['lua/functions', 'Lua guide', 'Sgt. Ready', 'Functions, callbacks, and what to do when a call fails', 'Small helpers, multiple returns, pcall, modules and predictable waits.'],
  ['', 'Getting started', 'Capt. Network', 'Your desk is the one with the blinking router', 'Run this first. We can argue about your strategy once it actually runs.'],
  ['world', 'Getting started', 'Maj. Packet', 'Read the map before you touch the switches', 'The routers are immune. The machines behind them are having a worse day.'],
  ['scoring', 'Getting started', 'Col. Scoreboard', 'An unplugged network is worth very little', '600 for service. 250 for security. 150 for putting things back.'],
  ['network', 'Network basics', 'Capt. Network', 'Two routers. One cable. Start there.', 'A small TOML network you can keep in your head.'],
  ['ticks', 'Network basics', 'Lt. Clock', 'Tick 10. Not “about ten seconds.”', 'Three ticks after seven is ten. Please stop bringing a stopwatch.'],
  ['sandbox', 'Network basics', 'Sgt. Sandbox', 'No, you cannot have a shell', 'You can have tables, modules, and a perfectly serviceable print function.'],
  ['tasks', 'Taking action', 'Lt. Task', 'Five slots. The sixth request goes back.', 'A task ID is a receipt, not a finished job.'],
  ['actions/scans', 'Taking action', 'Sgt. Scanner', 'CLEAN, as of when?', 'Six scans. Different questions. Every answer has a date on it.'],
  ['actions/shields', 'Taking action', 'Maj. Firewall', 'Who actually needs this connection?', 'Block the threat. Leave the server reachable. Both parts matter.'],
  ['actions/recovery', 'Taking action', 'Capt. Restore', 'Please isolate it before sending it to me', 'Twenty ticks to restore. Infection stays when you only pull the plug.'],
  ['strategy', 'Taking action', 'Col. Debug', 'One good run proves very little', 'Bring the seed, the score, and the tick where your plan fell apart.'],
  ['messaging', 'Teamwork', 'Lt. Relay', 'Delivered is not the same as understood', 'Register a handler, put a tick on your evidence, then send it.'],
  ['examples/sensing', 'Example agents', 'Sgt. Scanner', 'Attachment: six scans, no guesswork', 'A complete read-only agent, with each result printed.'],
  ['examples/shields', 'Example agents', 'Maj. Firewall', 'Attachment: let one workstation talk only to its server', 'Default deny, an exception, isolation and release.'],
  ['examples/recovery', 'Example agents', 'Capt. Restore', 'Attachment: off, on, and actually cured', 'A power experiment with a twenty-tick restore.'],
  ['examples/solo', 'Example agents', 'OCdt Gerami', 'Attachment: a local patrol with no friends', 'A complete solo scan, isolate, restore and release loop.'],
  ['examples/warning', 'Example agents', 'Lt. Relay', 'Attachment: warn the next LAN before the trouble arrives', 'Confirm a source, report its ID, discover it at the receiver, block transit.'],
  ['examples/leader', 'Example agents', 'Capt. Identity', 'Attachment: one leader, several owners', 'Elect the lowest agent ID; orders still execute at the owning router.'],
  ['examples/defenders', 'Example agents', 'Col. Debug', 'Ostrich, Basic, Coordinated, Advanced: bring a benchmark', 'Runnable baselines, scale fixtures and honest limits.'],
  ['networks/small', 'Network Samples', 'Capt. Network', 'Small networks you can draw on a napkin', 'One, two and three LANs. Copy the whole file.'],
  ['networks/scale', 'Network Samples', 'OCdt Syed', 'A thousand nodes without a thousand typing mistakes', 'Generate explicit TOML for 100, 1,000 or 10,000 total nodes.'],
  ['networks/agent', 'Network Samples', 'Sgt. Sandbox', 'Network Agent.md — instructions for a topology assistant', 'An LLM may generate network data. Keep competition agent code out of that prompt.'],
  ['debug/api', 'Debug workshop', 'Capt. Runtime', 'dbgguard: draw what your agent thinks it knows', '--dbg enables seven write-only visualization calls.'],
  ['debug/reconstruction', 'Debug workshop', 'Sgt. Scanner', 'Attachment: discovery beside live assessments', 'Two views, the same evidence, and no borrowed engine truth.'],
  ['api', 'Lua API', 'Capt. Runtime', 'The function index. Keep this one.', '23 calls, grouped by job. Exact signatures in the linked emails.'],
  ['api/identity', 'Lua API', 'Capt. Identity', 'That is a router number, not an agent number', 'They are both integers. This does not make them interchangeable.'],
  ['api/tasks', 'Lua API', 'Lt. Task', 'Your job receipts and where to file them', 'Start work, check its state, collect the result. On your router only.'],
  ['api/readiness', 'Lua API', 'Sgt. Ready', '“Finished” was not my question', 'A failed task is finished too. Check whether it succeeded.'],
  ['api/communication', 'Lua API', 'Lt. Relay', 'For the record: tx, rx, pub, sub, unsub', 'A receipt confirms acceptance. If you need a reply, write a reply.'],
  ['api/time', 'Lua API', 'Lt. Clock', 'Fuel spent. Ticks elapsed. Different columns.', 'Waiting twenty ticks does not mean paying twenty admission charges.'],
  ['constants', 'Quick reference', 'Sgt. Constants', 'Names and durations. No substitutions.', 'API 1.3. Five slots. RESTORE: 20 ticks. The rest is attached.'],
  ['errors', 'Quick reference', 'Col. Debug', 'Forward me the error, not “it broke”', 'The prefix is useful. So are the task ID and the tick.']
];

const sources = import.meta.glob('../../content/mail/*.md', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;
const attachments = import.meta.glob('../../../vanguard/{agents,networks}/**/*.{lua,toml,py,md}', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;
import { zipSync, strToU8 } from 'fflate';
import defaultNetwork from '../../../network.toml?raw';

function dataUrl(type: string, bytes: Uint8Array) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return `data:${type};base64,${btoa(binary)}`;
}
// Embed the real exercise files in both exports, including the offline HTML.
// Keeping the directory structure makes the printed commands usable after extraction.
const exerciseFiles = Object.fromEntries(Object.entries(attachments).map(([path, text]) => [path.replace('../../../', ''), strToU8(text)]));
exerciseFiles['network.toml'] = strToU8(defaultNetwork);
exerciseFiles['README.txt'] = strToU8('Extract this entire ZIP into an exercise folder. Run the handbook commands from that folder with the supplied vanguard executable. Keep the vanguard/ directories intact: defender entrypoints need patrol.lua beside them. Network Agent.md is in vanguard/networks/. No repository checkout is needed.\n');
export const exerciseDownload = dataUrl('application/zip', zipSync(exerciseFiles));
const renderer = new Renderer();
const escape = (s: string) => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
renderer.code = ({ text, lang }) => {
  const language = lang?.split(/\s/)[0] || 'text';
  const code = hljs.getLanguage(language) ? hljs.highlight(text, { language }).value : escape(text);
  return `<div class="code-block"><div class="code-toolbar"><span>${escape(language)}</span><button type="button" class="copy-code" aria-label="Copy ${escape(language)} code">Copy code</button></div><pre><code class="hljs language-${escape(language)}">${code}</code></pre></div>`;
};
renderer.heading = ({ tokens, depth }) => {
  const text = renderer.parser.parseInline(tokens);
  const id = text.replace(/<[^>]*>/g, '').toLowerCase().replace(/[^\w]+/g, '-').replace(/^-|-$/g, '');
  return `<h${depth} id="${id}">${text}</h${depth}>`;
};
export const emails = lessons.map(([slug, folder, sender, subject, preview], index) => {
  const raw = sources[`../../content/mail/${slug.replaceAll('/', '-') || 'welcome'}.md`];
  const body = raw?.replace('<!-- directory -->', folders.map(folder => `## ${folder}\n\n${lessons.filter(([s,f])=>f===folder && s!=='briefing').map(([s,,,subject,preview])=>`- [${subject}](/docs/${s ? `${s}/` : ''}) — ${preview}`).join('\n')}`).join('\n\n')).replace(/<!-- attachment:([\w./ -]+) -->/g, (_, path: string) => {
    const text = attachments[`../../../vanguard/${path}`];
    if (!text) throw new Error(`Missing attachment: ${path}`);
    const ext = path.split('.').at(-1);
    const filename = path.split('/').at(-1)!;
    const download = dataUrl('text/plain;charset=utf-8', strToU8(text));
    return `\n\n<a class="attachment-download" href="${download}" download="${escape(filename)}">Download ${escape(filename)}</a>\n\n\`\`\`${ext === 'py' ? 'python' : ext === 'md' ? 'text' : ext}\n${text.trimEnd()}\n\`\`\`\n`;
  });
  if (!body) throw new Error(`Missing email: ${slug}`);
  return { slug, folder, sender, subject, preview, body, ...voices[sender], order: index + 1,
    href: `/docs/${slug ? `${slug}/` : ''}`, html: marked.parse(body, { async: false, renderer }),
    minutes: Math.max(1, Math.ceil(body.split(/\s+/).length / 180)) };
});
export type Email = typeof emails[number];
