import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/tests/fixtures/index.html");
  await page.getByRole("button", { name: "Open PDF", exact: true }).click();
  await expect(page.locator("[data-page]")).toHaveCount(12);
  await page.getByTitle("Fit width", { exact: true }).click();
  await expect(page.locator('[data-page="1"] .textLayer')).toContainText("Page 1:");
});

test("wheel scrolling stays put; page jumps, zoom and resizing render correctly", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const reader = page.getByLabel("PDF pages");
  const height = await reader.evaluate((el) => el.scrollHeight);
  const thirdPageTop = await page.locator('[data-page="3"]').evaluate((el) => (el as HTMLElement).offsetTop);
  await reader.hover();
  await page.mouse.wheel(0, thirdPageTop + 80);
  await expect(page.getByText("3 / 12", { exact: true })).toBeVisible();
  const top = await reader.evaluate((el) => el.scrollTop);
  await page.waitForTimeout(1000);
  expect(await reader.evaluate((el) => el.scrollTop)).toBeCloseTo(top, 0);
  expect(await reader.evaluate((el) => el.scrollHeight)).toBe(height);
  await page.getByRole("button", { name: "›", exact: true }).click();
  await expect(page.getByText("4 / 12", { exact: true })).toBeVisible();
  await expect(page.getByTitle("Fit width", { exact: true })).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator('[data-page="4"] .textLayer')).toContainText("Page 4:");
  await page.setViewportSize({ width: 1440, height: 920 });
  await page.getByTitle("Zoom in", { exact: true }).click({ clickCount: 4 });
  await page.getByTitle("Fit page", { exact: true }).click();
  await expect.poll(() => page.locator('[data-page="4"] canvas').evaluate((el) => (el as HTMLCanvasElement).width)).toBeGreaterThan(0);
  await expect(page.locator('[data-page="4"] .textLayer')).toContainText("Page 4:");
  await expect(page.getByRole("alert")).toHaveCount(0);
  expect(errors).toEqual([]);
});

test("earlier citations retain their own page", async ({ page }) => {
  const input = page.getByPlaceholder("Ask anything...");
  await input.fill("First question");
  await input.press("Enter");
  await expect(page.getByText("Answer 1 [1].", { exact: false })).toBeVisible();
  await input.fill("Follow up");
  await input.press("Enter");
  await expect(page.getByText("Answer 2 [1].", { exact: false })).toBeVisible();
  await page.getByRole("button", { name: "p.2", exact: true }).click();
  await expect(page.getByText("2 / 12", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "p.8", exact: true }).click();
  await expect(page.getByText("8 / 12", { exact: true })).toBeVisible();
  await expect(page.locator('[data-page="8"] .textLayer')).toContainText("Page 8:");
  const highlightTop = await page.locator('[data-page="8"] [style*="flash-amber"]').evaluate((el) => (el as HTMLElement).offsetTop);
  expect(highlightTop).toBeLessThan(130);
  await page.screenshot({ path: "test-results/reader-citations.png" });
});

test("outline toggling keeps the reading position and panels within the window", async ({ page }) => {
  await page.setViewportSize({ width: 1224, height: 800 });
  const reader = page.getByLabel("PDF pages");
  const position = () => reader.evaluate((el) => {
    const first = el.querySelector<HTMLElement>('[data-page="1"]')!;
    return { fraction: (el.scrollTop + 16 - first.offsetTop) / first.offsetHeight,
      left: el.scrollLeft, width: el.clientWidth };
  });
  for (const fit of [false, true]) {
    if (!fit) await page.getByTitle("Zoom in", { exact: true }).click();
    else await page.getByTitle("Fit width", { exact: true }).click();
    await expect.poll(async () => (await position()).width).toBeGreaterThan(0);
    await page.waitForTimeout(200);
    await reader.evaluate((el) => { el.scrollTop = 220; el.scrollLeft = 120; });
    await page.waitForTimeout(100);
    const before = await position();
    await page.getByTitle("Toggle outline", { exact: true }).click();
    await expect.poll(async () => (await position()).width).toBeGreaterThan(before.width);
    await expect.poll(async () => Math.abs((await position()).fraction - before.fraction)).toBeLessThan(0.003);
    const collapsed = await page.getByTitle("Show outline", { exact: true }).boundingBox();
    const readerBox = await reader.boundingBox();
    expect(collapsed!.y).toBe(readerBox!.y);
    expect(collapsed!.height).toBe(readerBox!.height);
    expect(collapsed!.x + collapsed!.width).toBeLessThanOrEqual(readerBox!.x);
    await page.screenshot({ path: "test-results/reader-outline-collapsed.png" });
    await page.getByTitle("Toggle outline", { exact: true }).click();
    await expect.poll(async () => (await position()).width).toBe(before.width);
    await expect.poll(async () => Math.abs((await position()).fraction - before.fraction)).toBeLessThan(0.003);
    await expect(page.getByTitle("Fit width", { exact: true })).toHaveAttribute("aria-pressed", String(fit));
    if (fit) await expect.poll(() => reader.evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    const group = await page.locator('[data-panel-group]').boundingBox();
    expect(group!.x + group!.width).toBeLessThanOrEqual(1224);
    const send = await page.getByRole("button", { name: "Send", exact: true }).boundingBox();
    expect(send!.x + send!.width).toBeLessThanOrEqual(1224);
  }
  await page.screenshot({ path: "test-results/reader-outline.png" });
});

test("IME confirmation does not send a question", async ({ page }) => {
  const input = page.getByPlaceholder("Ask anything...");
  await input.fill("日本語の入力");
  await input.dispatchEvent("keydown", { key: "Enter", code: "Enter", isComposing: true });
  await expect(page.locator(".ai-markdown")).toHaveCount(0);
  await expect(input).toHaveValue("日本語の入力");
});
