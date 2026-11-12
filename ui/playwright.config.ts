import { defineConfig } from '@playwright/test';

export default defineConfig({
	testDir: './tests',
	fullyParallel: false,
	workers: 1,
	timeout: 120_000,
	use: {
		baseURL: 'http://127.0.0.1:18124',
		viewport: { width: 1440, height: 1000 },
		colorScheme: 'light',
		launchOptions: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE
			? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE }
			: undefined
	},
	webServer: {
		command: 'cargo run -- --script vanguard/agents/defender.lua --network network.toml --port 18124',
		cwd: '..',
		url: 'http://127.0.0.1:18124/api/health',
		timeout: 180_000,
		reuseExistingServer: false
	}
});
