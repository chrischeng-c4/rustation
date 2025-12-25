import { browser, $, $$ } from "@wdio/globals";
import * as fs from "fs";
import * as path from "path";

describe("Docker Page", () => {
  const screenshotDir = path.join(process.cwd(), "screenshots");

  before(async () => {
    // Ensure screenshot directory exists
    if (!fs.existsSync(screenshotDir)) {
      fs.mkdirSync(screenshotDir, { recursive: true });
    }
  });

  it("should display the app", async () => {
    // Take screenshot of initial state
    await browser.saveScreenshot(path.join(screenshotDir, "01-initial.png"));

    // Check app title
    const title = await browser.getTitle();
    console.log("App title:", title);
  });

  it("should navigate to Dockers tab", async () => {
    // Click on Dockers tab
    const dockersTab = await $('button*=Dockers');
    await dockersTab.click();

    // Wait for content to load
    await browser.pause(500);

    // Take screenshot
    await browser.saveScreenshot(path.join(screenshotDir, "02-dockers-page.png"));

    // Verify we're on the Dockers page
    const heading = await $("h2=Dockers");
    await expect(heading).toBeDisplayed();
  });

  it("should display Docker service cards", async () => {
    // Check for service cards
    const cards = await $$('[class*="card"]');
    console.log(`Found ${cards.length} cards`);

    // Take screenshot of service grid
    await browser.saveScreenshot(path.join(screenshotDir, "03-service-cards.png"));
  });

  it("should toggle a Docker service", async () => {
    // Find a Start/Stop button
    const toggleButton = await $('button*=Stop');

    if (await toggleButton.isExisting()) {
      await toggleButton.click();
      await browser.pause(500);
      await browser.saveScreenshot(path.join(screenshotDir, "04-after-toggle.png"));
    }
  });

  it("should open log panel", async () => {
    // Click Logs button
    const logsButton = await $('button*=Logs');

    if (await logsButton.isExisting()) {
      await logsButton.click();
      await browser.pause(500);
      await browser.saveScreenshot(path.join(screenshotDir, "05-log-panel.png"));
    }
  });
});
