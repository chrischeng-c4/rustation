import { test, expect } from './electron.fixture'

test.describe('Environment Management', () => {
  test('should navigate to Env page when clicking Env button', async ({ page }) => {
    // Wait for app to load
    await page.waitForSelector('[data-testid="project-tabs"]', { timeout: 10000 }).catch(() => {})

    // Look for the Env button in the worktree row
    const envButton = page.getByRole('button', { name: /Env/i })

    // If no project is open, Env button won't be visible
    const isVisible = await envButton.isVisible().catch(() => false)

    if (isVisible) {
      await envButton.click()

      // Should show Environment heading
      await expect(page.locator('h2', { hasText: 'Environment' })).toBeVisible({ timeout: 5000 })
    } else {
      // No project open - this is expected in clean state
      test.skip(true, 'No project open - Env button not visible')
    }
  })

  test('should display Env page elements when navigated', async ({ page }) => {
    // Wait for app to load
    await page.waitForTimeout(2000)

    const envButton = page.getByRole('button', { name: /Env/i })
    const isVisible = await envButton.isVisible().catch(() => false)

    if (!isVisible) {
      test.skip(true, 'No project open - Env button not visible')
      return
    }

    await envButton.click()

    // Verify key UI elements
    await expect(page.locator('h2', { hasText: 'Environment' })).toBeVisible({ timeout: 5000 })

    // Should have Auto-Copy toggle button
    await expect(page.getByRole('button', { name: /Auto-Copy/i })).toBeVisible()

    // Should have Manual Sync section
    await expect(page.locator('h3', { hasText: 'Manual Sync' })).toBeVisible()

    // Should have Configuration section
    await expect(page.locator('h3', { hasText: 'Configuration' })).toBeVisible()

    // Should have Recent Activity section
    await expect(page.locator('h3', { hasText: 'Recent Activity' })).toBeVisible()
  })
})
