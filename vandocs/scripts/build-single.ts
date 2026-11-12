import { build } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { resolve } from 'node:path';

await build({
  configFile: false,
  plugins: [tailwindcss(), svelte()],
  resolve: { alias: { $lib: resolve('src/lib') } },
  build: {
    outDir: '.single-build', emptyOutDir: true, cssCodeSplit: false,
    lib: { entry: resolve('standalone/entry.ts'), name: 'VanMail', formats: ['iife'], fileName: () => 'mail.js', cssFileName: 'mail' },
  },
});
const js = (await readFile('.single-build/mail.js', 'utf8')).replaceAll('</script', '<\\/script');
const css = (await readFile('.single-build/mail.css', 'utf8')).replaceAll('</style', '<\\/style');
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>VanMail</title><style>${css}</style></head><body><div id="app"></div><noscript>Enable JavaScript to open the offline VanMail inbox.</noscript><script>${js}</script></body></html>`;
// out/ is the generated distribution directory. Remove older multi-page exports.
await rm('out', { recursive: true, force: true });
await mkdir('out', { recursive: true });
await writeFile('out/VanMail.html', html);
console.log(`Standalone handbook: out/VanMail.html (${Math.ceil(Buffer.byteLength(html)/1024)} KiB, no external assets)`);
