const file = Bun.file(new URL('../out/VanMail.html', import.meta.url));
if (!await file.exists()) throw new Error('Build the handbook first with bun run build.');
const server = Bun.serve({
  hostname: '127.0.0.1',
  port: 3001,
  fetch(request) {
    const path = new URL(request.url).pathname;
    return path === '/' || path === '/VanMail.html'
      ? new Response(file, { headers: { 'Content-Type': 'text/html; charset=utf-8' } })
      : new Response('Not found', { status: 404 });
  }
});
console.log(`VanMail single-file preview: ${server.url}`);
