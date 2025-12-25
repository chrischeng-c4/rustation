import { test, expect } from "@playwright/test";

// Helper to click tab using data-slot attribute
async function clickTab(page: import("@playwright/test").Page, tabName: string) {
  await page.locator(`[data-slot="tabs-trigger"]`, { hasText: tabName }).click();
  await page.waitForTimeout(100); // Small delay for UI update
}

test.describe("Docker Page", () => {
  test("should display initial app state", async ({ page }) => {
    await page.goto("/");

    // Screenshot initial state
    await page.screenshot({ path: "screenshots/01-initial.png", fullPage: true });

    // Verify app title
    await expect(page.locator("h1")).toContainText("rustation");
  });

  test("should navigate to Dockers tab", async ({ page }) => {
    await page.goto("/");

    // Click Dockers tab using data-slot selector
    await clickTab(page, "Dockers");

    // Screenshot Dockers page
    await page.screenshot({ path: "screenshots/02-dockers-page.png", fullPage: true });

    // Verify heading
    await expect(page.getByRole("heading", { name: "Dockers" })).toBeVisible();
  });

  test("should display Docker service cards", async ({ page }) => {
    await page.goto("/");
    await clickTab(page, "Dockers");

    // Check for service cards using data-slot
    const cards = page.locator('[data-slot="card"]');
    await expect(cards.first()).toBeVisible();

    // Should have 6 built-in services
    const count = await cards.count();
    console.log(`Found ${count} service cards`);
    expect(count).toBe(6);

    // Screenshot service grid
    await page.screenshot({ path: "screenshots/03-service-cards.png", fullPage: true });
  });

  test("should show status indicators", async ({ page }) => {
    await page.goto("/");
    await clickTab(page, "Dockers");

    // Check for status badges
    const runningBadge = page.getByText("Running").first();
    const stoppedBadge = page.getByText("Stopped").first();

    // At least one should be visible
    const hasRunning = await runningBadge.isVisible().catch(() => false);
    const hasStopped = await stoppedBadge.isVisible().catch(() => false);

    expect(hasRunning || hasStopped).toBeTruthy();
  });

  test("should toggle Docker service", async ({ page }) => {
    await page.goto("/");
    await clickTab(page, "Dockers");

    // Find and click Stop button
    const stopButton = page.getByRole("button", { name: /Stop/i }).first();

    if (await stopButton.isVisible()) {
      await stopButton.click();
      await page.waitForTimeout(500);

      // Screenshot after toggle
      await page.screenshot({ path: "screenshots/04-after-toggle.png", fullPage: true });
    }
  });

  test("should open log panel", async ({ page }) => {
    await page.goto("/");
    await clickTab(page, "Dockers");

    // Click Logs button
    const logsButton = page.getByRole("button", { name: /Logs/i }).first();

    if (await logsButton.isVisible()) {
      await logsButton.click();
      await page.waitForTimeout(500);

      // Screenshot log panel
      await page.screenshot({ path: "screenshots/05-log-panel.png", fullPage: true });

      // Verify sheet is open
      await expect(page.getByText("Container output logs")).toBeVisible();
    }
  });

  test("should navigate between all tabs", async ({ page }) => {
    await page.goto("/");

    // Tasks tab (default)
    await expect(page.getByRole("heading", { name: "Tasks" })).toBeVisible();
    await page.screenshot({ path: "screenshots/06-tasks-tab.png", fullPage: true });

    // Dockers tab
    await clickTab(page, "Dockers");
    await expect(page.getByRole("heading", { name: "Dockers" })).toBeVisible();

    // Settings tab
    await clickTab(page, "Settings");
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await page.screenshot({ path: "screenshots/07-settings-tab.png", fullPage: true });
  });
});
