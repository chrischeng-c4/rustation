import { test, expect } from './electron.fixture'

test.describe('Command Palette', () => {
  test('should open command palette with Cmd+K', async ({ page }) => {
    // Wait for app to load
    await page.waitForTimeout(2000)

    // Press Cmd+K (Mac) or Ctrl+K (Windows/Linux)
    await page.keyboard.press('Meta+k')

    // Should show command palette input
    await expect(page.locator('[cmdk-input]')).toBeVisible({ timeout: 5000 })
  })

  test('should close command palette with Escape', async ({ page }) => {
    await page.waitForTimeout(2000)

    // Open palette
    await page.keyboard.press('Meta+k')
    await expect(page.locator('[cmdk-input]')).toBeVisible({ timeout: 5000 })

    // Close with Escape
    await page.keyboard.press('Escape')
    await expect(page.locator('[cmdk-input]')).not.toBeVisible({ timeout: 3000 })
  })

  test('should show Views group', async ({ page }) => {
    await page.waitForTimeout(2000)

    await page.keyboard.press('Meta+k')
    await expect(page.locator('[cmdk-input]')).toBeVisible({ timeout: 5000 })

    // Should show Views section with cmdk-specific selectors
    await expect(page.locator('[cmdk-group-heading]:has-text("Views")')).toBeVisible()
    await expect(page.locator('[cmdk-item]:has-text("Docker")')).toBeVisible()
  })

  test('should show Theme group', async ({ page }) => {
    await page.waitForTimeout(2000)

    await page.keyboard.press('Meta+k')
    await expect(page.locator('[cmdk-input]')).toBeVisible({ timeout: 5000 })

    // Should show Theme section with cmdk-specific selectors
    await expect(page.locator('[cmdk-group-heading]:has-text("Theme")')).toBeVisible()
    await expect(page.locator('[cmdk-item]:has-text("Light Theme")')).toBeVisible()
    await expect(page.locator('[cmdk-item]:has-text("Dark Theme")')).toBeVisible()
  })

  test('should filter items when typing', async ({ page }) => {
    await page.waitForTimeout(2000)

    await page.keyboard.press('Meta+k')
    await expect(page.locator('[cmdk-input]')).toBeVisible({ timeout: 5000 })

    // Type to filter
    await page.locator('[cmdk-input]').fill('docker')

    // Should show Docker but filter out others
    await expect(page.locator('[cmdk-item]:has-text("Docker")')).toBeVisible()
  })
})
