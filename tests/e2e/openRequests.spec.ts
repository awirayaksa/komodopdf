import { expect, test } from "@playwright/test";

test("an association cold start opens its PDF without using the file dialog", async ({ page }) => {
  const path = "C:\\日本語 資料\\100%20 #1.pdf";
  await page.addInitScript((path) => { (window as any).__initialPdfPaths = [path]; }, path);
  await page.goto("/tests/fixtures/index.html");
  await expect(page.locator("[data-page]")).toHaveCount(12, { timeout: 15000 });
  await expect(page.locator('[data-page="1"] .textLayer')).toContainText("Page 1:");
  expect(await page.evaluate(() => (window as any).__readPdfPaths)).toEqual([path]);
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("running-app requests recover from a missing file and open multiple paths in order", async ({ page }) => {
  await page.goto("/tests/fixtures/index.html");
  const paths = ["C:\\日本語 資料\\first file.pdf", "C:\\missing.pdf", "\\\\server\\share\\最後の 資料.pdf"];
  await page.evaluate((paths) => (window as any).__openNativePdfs(paths), paths);
  await expect(page.locator("[data-page]")).toHaveCount(12, { timeout: 15000 });
  await expect.poll(() => page.evaluate(() => (window as any).__readerStore.getState().fileName)).toBe(paths[2]);
  expect(await page.evaluate(() => (window as any).__readPdfPaths)).toEqual(paths);
  // A later launch remains usable after the earlier error.
  await page.evaluate(() => (window as any).__openNativePdfs(["C:\\later.pdf"]));
  await expect.poll(() => page.evaluate(() => (window as any).__readerStore.getState().fileName)).toBe("C:\\later.pdf");
  await expect(page.getByRole("alert")).toHaveCount(0);
});
