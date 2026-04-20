import { defineConfig } from "@playwright/test";

export default defineConfig({
	testDir: "./e2e",
	timeout: 30000,
	retries: 1,
	use: {
		baseURL: "http://localhost:1420",
	},
	webServer: {
		command: "pnpm tauri dev",
		port: 1420,
		reuseExistingServer: true,
	},
});
