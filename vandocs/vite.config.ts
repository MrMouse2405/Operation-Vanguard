import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  resolve: { alias: { $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) } },
  // Route-based output is an internal fixture for the browser regression suite.
  plugins: [tailwindcss(), sveltekit({ adapter: adapter({ pages: '.site-build', assets: '.site-build' }) })]
});
