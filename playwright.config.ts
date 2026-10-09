import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "tests/e2e", workers: 1,
  use: { baseURL: "http://localhost:1420", viewport: { width: 1280, height: 900 }, trace: "retain-on-failure" },
  webServer: { command: "npm run dev", url: "http://localhost:1420", reuseExistingServer: !process.env.CI },
});
