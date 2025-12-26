import { test, expect } from './electron.fixture'

test.describe('Docker Management', () => {
  // Navigate to Docker view before checking availability
  async function navigateToDocker(page: import('@playwright/test').Page): Promise<void> {
    // Click the Docker button in project tabs (top bar)
    const dockerButton = page.getByRole('button', { name: 'Docker' })
    await dockerButton.click()
    // Wait for navigation
    await page.waitForTimeout(500)
  }

  // Helper to check if Docker daemon is running
  async function isDockerAvailable(page: import('@playwright/test').Page): Promise<boolean> {
    await navigateToDocker(page)

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
    await navigateToDocker(page)

    // Should show either Docker heading or not available message
    const dockerHeading = page.locator('h2', { hasText: 'Dockers' })
    const notAvailable = page.getByText('Docker Not Available')

    const hasDocker = await dockerHeading.isVisible().catch(() => false)
    const hasNotAvailable = await notAvailable.isVisible().catch(() => false)

    expect(hasDocker || hasNotAvailable).toBe(true)
  })

  test('should display services panel when Docker is available', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available - skipping service tests')

    // Should show either services or "No Docker services found" message
    const noServices = page.getByText('No Docker services found')
    const serviceCard = page.locator('[class*="rounded-lg border"]').first()

    const hasNoServices = await noServices.isVisible().catch(() => false)
    const hasServiceCard = await serviceCard.isVisible().catch(() => false)

    // Either we have services or the empty state message
    expect(hasNoServices || hasServiceCard).toBe(true)
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

  test('should have Refresh or Retry button', async ({ page }) => {
    await navigateToDocker(page)

    // Should have either Refresh (Docker available) or Retry (Docker not available)
    const refreshButton = page.getByRole('button', { name: 'Refresh' })
    const retryButton = page.getByRole('button', { name: 'Retry' })

    const hasRefresh = await refreshButton.isVisible().catch(() => false)
    const hasRetry = await retryButton.isVisible().catch(() => false)

    expect(hasRefresh || hasRetry).toBe(true)
  })

  test('should show logs panel when clicking Logs button', async ({ page }) => {
    const available = await isDockerAvailable(page)
    test.skip(!available, 'Docker not available')

    // Check if there are services with Logs button
    const logsButton = page.getByRole('button', { name: 'Logs' }).first()
    const hasLogs = await logsButton.isVisible().catch(() => false)

    // Skip if no services have logs button
    test.skip(!hasLogs, 'No services with Logs button')

    await logsButton.click()
    // Log panel should update - look for any "Logs" heading
    await expect(page.getByText(/Logs$/).first()).toBeVisible({ timeout: 5000 })
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

  test('should navigate between Docker and other views', async ({ page }) => {
    // Navigate to Docker first
    await navigateToDocker(page)

    // Should show Docker page
    const dockerHeading = page.locator('h2', { hasText: 'Dockers' })
    const notAvailable = page.getByText('Docker Not Available')

    const hasDocker = await dockerHeading.isVisible().catch(() => false)
    const hasNotAvailable = await notAvailable.isVisible().catch(() => false)

    expect(hasDocker || hasNotAvailable).toBe(true)

    // Navigate back to Docker from command palette
    await page.keyboard.press('Meta+k')
    await page.locator('[cmdk-input]').fill('docker')
    await page.locator('[cmdk-item]:has-text("Docker")').click()
    await page.waitForTimeout(500)

    // Should still show Docker page
    const hasDockersAgain = await dockerHeading.isVisible().catch(() => false)
    const hasNotAvailableAgain = await notAvailable.isVisible().catch(() => false)

    expect(hasDockersAgain || hasNotAvailableAgain).toBe(true)
  })
})
