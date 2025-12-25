import { test, expect } from './electron.fixture'

test.describe('Docker Management', () => {
  // Helper to check if Docker is available
  async function isDockerAvailable(page: import('@playwright/test').Page): Promise<boolean> {
    // Wait for either Docker content or "Docker Not Available" message
    const dockerHeading = page.locator('h2', { hasText: 'Dockers' })
    const notAvailable = page.getByText('Docker Not Available')

    await Promise.race([
      dockerHeading.waitFor({ timeout: 10000 }).catch(() => {}),
      notAvailable.waitFor({ timeout: 10000 }).catch(() => {}),
    ])

    return await dockerHeading.isVisible()
  }

  test('should display the app window', async ({ page }) => {
    // Check that the window is visible
    const title = await page.title()
    expect(title).toBe('rstn')
  })

  test('should show Docker page (available or not available)', async ({ page }) => {
    const available = await isDockerAvailable(page)

    if (available) {
      // Docker is available - should show services heading
      await expect(page.locator('h2', { hasText: 'Dockers' })).toBeVisible()
    } else {
      // Docker not available - should show not available message
      await expect(page.getByText('Docker Not Available')).toBeVisible()
      await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible()
    }
  })

  test('should display services when Docker is available', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available - skipping service tests')

    // Should show at least one service or empty state
    const servicesPanel = page.locator('.space-y-3')
    await expect(servicesPanel).toBeVisible({ timeout: 5000 })
  })

  test('should show service status badges when Docker is available', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available')

    // Check for status badges by looking for Running or Stopped text
    const runningBadge = page.getByText('Running').first()
    const stoppedBadge = page.getByText('Stopped').first()

    const hasRunning = await runningBadge.isVisible().catch(() => false)
    const hasStopped = await stoppedBadge.isVisible().catch(() => false)

    // At least one status should be visible if there are services
    const noServices = await page.getByText('No Docker services found').isVisible().catch(() => false)
    if (!noServices) {
      expect(hasRunning || hasStopped).toBe(true)
    }
  })

  test('should have Start/Stop buttons when Docker is available', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available')

    // Should have either Start or Stop button (or no services)
    const startButton = page.getByRole('button', { name: 'Start' }).first()
    const stopButton = page.getByRole('button', { name: 'Stop' }).first()

    const hasStart = await startButton.isVisible().catch(() => false)
    const hasStop = await stopButton.isVisible().catch(() => false)

    const noServices = await page.getByText('No Docker services found').isVisible().catch(() => false)
    if (!noServices) {
      expect(hasStart || hasStop).toBe(true)
    }
  })

  test('should have Refresh button', async ({ page }) => {
    const available = await isDockerAvailable(page)

    if (available) {
      const refreshButton = page.getByRole('button', { name: 'Refresh' })
      await expect(refreshButton).toBeVisible()
    } else {
      // When Docker not available, there's a Retry button
      const retryButton = page.getByRole('button', { name: 'Retry' })
      await expect(retryButton).toBeVisible()
    }
  })

  test('should show logs panel when clicking Logs button', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available')

    // Find and click the Logs button if services exist
    const logsButton = page.getByRole('button', { name: 'Logs' }).first()
    const hasLogs = await logsButton.isVisible().catch(() => false)

    if (hasLogs) {
      await logsButton.click()
      // Log panel should update - check for the service name in the panel title
      // The title format is "[ServiceName] Logs"
      await expect(page.getByText('PostgreSQL Logs')).toBeVisible({ timeout: 5000 })
    }
  })

  test('should have Copy URL button on service cards', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available')

    // Check for Copy URL button if services exist
    const copyButton = page.getByRole('button', { name: 'Copy URL' }).first()
    const hasCopy = await copyButton.isVisible().catch(() => false)

    const noServices = await page.getByText('No Docker services found').isVisible().catch(() => false)
    if (!noServices) {
      expect(hasCopy).toBe(true)
    }
  })

  test('should navigate between tabs', async ({ page }) => {
    // Wait for initial load
    await page.waitForTimeout(500)

    // Click Tasks in sidebar navigation
    const tasksNav = page.getByText('Tasks').first()
    await tasksNav.click()

    // Check for tasks content
    await expect(page.locator('h2', { hasText: 'Tasks' })).toBeVisible()

    // Click Settings in sidebar
    const settingsNav = page.getByText('Settings').first()
    await settingsNav.click()

    // Check for settings content (Coming Soon text)
    await expect(page.getByText('Settings - Coming Soon')).toBeVisible()

    // Click Docker in sidebar to go back
    const dockerNav = page.getByText('Docker').first()
    await dockerNav.click()

    // Should show either Docker content or not available message
    const dockerHeading = page.locator('h2', { hasText: 'Dockers' })
    const notAvailable = page.getByText('Docker Not Available')

    const hasDocker = await dockerHeading.isVisible().catch(() => false)
    const hasNotAvailable = await notAvailable.isVisible().catch(() => false)

    expect(hasDocker || hasNotAvailable).toBe(true)
  })
})
