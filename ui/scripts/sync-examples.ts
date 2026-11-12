/** Vendor upstream documentation examples against a pinned, reviewable revision. */
import { readdir, mkdir, writeFile } from 'node:fs/promises';

const revision = '3c4152d7d3904e76f6f756ec8ffca7c9ec72f977';
const base = `https://raw.githubusercontent.com/huntabyte/shadcn-svelte/${revision}`;
const custom = new Set(['chart', 'sidebar', 'trace', 'message-scroller', 'empty', 'code-block']);
const names = (await readdir('src/lib/components/ui', { withFileTypes: true }))
	.filter((entry) => entry.isDirectory()).map((entry) => entry.name).sort();
await mkdir('src/lib/examples', { recursive: true });

for (const name of names.filter((name) => !custom.has(name))) {
	const response = await fetch(`${base}/docs/src/lib/registry/examples/${name}-demo.svelte`);
	if (!response.ok) throw new Error(`Missing upstream example: ${name} (${response.status})`);
	const source = (await response.text())
		.replaceAll('$lib/registry/ui/', '$lib/components/ui/')
		.replaceAll('$lib/registry/hooks/', '$lib/hooks/');
	await writeFile(`src/lib/examples/${name}.svelte`, `<!-- Adapted from shadcn-svelte ${revision}; MIT, see THIRD_PARTY.md. -->\n${source}`);
}
const license = await fetch(`${base}/LICENSE.md`);
if (!license.ok) throw new Error('Could not retrieve upstream license');
await writeFile('SHADCN-LICENSE.md', await license.text());
console.log(`Vendored ${names.length - custom.size} examples from ${revision}.`);
