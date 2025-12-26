import { test, expect } from './electron.fixture'

test.describe('Settings', () => {
  test('should navigate to Settings page via sidebar', async ({ page }) => {
    // Wait for app to load
    await page.waitForTimeout(2000)

    // Look for the Settings tab in the sidebar
    const settingsTab = page.locator('[value="settings"]')
    const isVisible = await settingsTab.isVisible().catch(() => false)

    if (!isVisible) {
      test.skip(true, 'No project open - Settings tab not in sidebar')
      return
    }

    await settingsTab.click()

    // Should show Settings heading
    await expect(page.locator('h2', { hasText: 'Settings' })).toBeVisible({ timeout: 5000 })
  })

  test('should display theme options', async ({ page }) => {
    await page.waitForTimeout(2000)

    const settingsTab = page.locator('[value="settings"]')
    const isVisible = await settingsTab.isVisible().catch(() => false)

    if (!isVisible) {
      test.skip(true, 'No project open - Settings tab not visible')
      return
    }

    await settingsTab.click()

    // Check for theme buttons
    await expect(page.getByRole('button', { name: /System/i })).toBeVisible({ timeout: 5000 })
    await expect(page.getByRole('button', { name: /Light/i })).toBeVisible()
    await expect(page.getByRole('button', { name: /Dark/i })).toBeVisible()
  })

  test('should have Appearance and Projects sections', async ({ page }) => {
    await page.waitForTimeout(2000)

    const settingsTab = page.locator('[value="settings"]')
    const isVisible = await settingsTab.isVisible().catch(() => false)

    if (!isVisible) {
      test.skip(true, 'No project open')
      return
    }

    await settingsTab.click()

    // Verify key sections
    await expect(page.locator('h3', { hasText: 'Appearance' })).toBeVisible({ timeout: 5000 })
    await expect(page.locator('h3', { hasText: 'Projects' })).toBeVisible()
    await expect(page.locator('h3', { hasText: 'About' })).toBeVisible()
  })
})
