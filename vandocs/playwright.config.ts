import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  workers: 1,
  use: { baseURL: 'http://127.0.0.1:18125', launchOptions: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE } : undefined },
  webServer: { command: 'python3 -m http.server 18125 --bind 127.0.0.1 --directory .site-build', url: 'http://127.0.0.1:18125/', reuseExistingServer: false },
});
