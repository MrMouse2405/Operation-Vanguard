import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? files(join(dir, e.name)) : e.name.endsWith('.md') ? [join(dir, e.name)] : []);
}
const pages = files('content/mail');
const content = pages.map(f => readFileSync(f, 'utf8').replace(/<!-- attachment:([\w./ -]+) -->/g, (_, path) => readFileSync(join('../vanguard', path), 'utf8'))).join('\n');
const manifest = readFileSync('src/lib/mail.ts', 'utf8');
for (const file of pages) {
  const name = file.replace('content/mail/', '').replace(/\.md$/, '');
  const slugs = [...manifest.matchAll(/^  \['([^']*)', '/gm)].map(m => m[1].replaceAll('/', '-') || 'welcome');
  if (!slugs.includes(name)) throw new Error(`Unlisted email: ${file}`);
}
if (/cargo (run|install|build)|bun (install|run build)/.test(content)) throw new Error('Internal build/install instructions leaked into competitor mail');
const spec = readFileSync('../src/api.lua', 'utf8');
const functions = [...spec.matchAll(/^function vanguard\.(\w+)\(/gm)].map(m => m[1]);
for (const name of functions) {
  if (!content.includes(`## vanguard.${name}\n`)) throw new Error(`Missing function reference: ${name}`);
}
const costs = spec.slice(spec.indexOf('vanguard.tick_cost ='));
for (const [,name,ticks] of costs.matchAll(/^\s+(\w+) = (\d+),/gm)) {
  if (!content.includes(`| ${name} | ${ticks} |`)) throw new Error(`Missing/mismatched duration: ${name}`);
}
for (const [,href] of content.matchAll(/\]\((\/docs[^)#]*)(?:#[^)]*)?\)/g)) {
  const slug = href.replace(/^\/docs\/?/, '').replace(/\/$/, '');
  const candidates = [`content/mail/${slug.replaceAll('/', '-') || 'welcome'}.md`];
  if (!candidates.some(c => pages.includes(c))) throw new Error(`Broken documentation link: ${href}`);
}
console.log(`${functions.length} functions, 19 action durations and internal links verified across ${pages.length} emails.`);
