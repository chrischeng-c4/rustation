import { test, expect } from './electron.fixture'

test.describe('Docker Management', () => {
  test('should display the app window', async ({ page }) => {
    // Check that the window is visible
    const title = await page.title()
    expect(title).toBe('rstn')
  })

  test('should show Docker tab with services', async ({ page }) => {
    // Check for the Dockers heading
    await expect(page.locator('h2', { hasText: 'Dockers' })).toBeVisible()

    // Check for service cards (look for card with service name)
    await expect(page.getByText('PostgreSQL').first()).toBeVisible()
  })

  test('should display all 6 built-in services', async ({ page }) => {
    // Check for all service names
    const services = ['PostgreSQL', 'MySQL', 'MongoDB', 'Redis', 'RabbitMQ', 'NATS']

    for (const service of services) {
      await expect(page.getByText(service).first()).toBeVisible()
    }
  })

  test('should show service status badges', async ({ page }) => {
    // Check for status badges by looking for Running or Stopped text
    const runningBadge = page.getByText('Running').first()
    const stoppedBadge = page.getByText('Stopped').first()

    const hasRunning = await runningBadge.isVisible().catch(() => false)
    const hasStopped = await stoppedBadge.isVisible().catch(() => false)

    expect(hasRunning || hasStopped).toBe(true)
  })

  test('should have Start/Stop buttons on service cards', async ({ page }) => {
    // Should have either Start or Stop button
    const startButton = page.getByRole('button', { name: 'Start' }).first()
    const stopButton = page.getByRole('button', { name: 'Stop' }).first()

    const hasStart = await startButton.isVisible().catch(() => false)
    const hasStop = await stopButton.isVisible().catch(() => false)

    expect(hasStart || hasStop).toBe(true)
  })

  test('should toggle service when clicking Start/Stop', async ({ page }) => {
    // Find the first service card with a Start button
    const startButton = page.getByRole('button', { name: 'Start' }).first()

    if (await startButton.isVisible()) {
      // Click Start
      await startButton.click()

      // Wait for state change - button should become Stop
      await expect(page.getByRole('button', { name: 'Stop' }).first()).toBeVisible({ timeout: 5000 })
    }
  })

  test('should have Refresh button', async ({ page }) => {
    const refreshButton = page.getByRole('button', { name: 'Refresh' })
    await expect(refreshButton).toBeVisible()
  })

  test('should open log sheet when clicking Logs button', async ({ page }) => {
    // Find and click the Logs button
    const logsButton = page.getByRole('button', { name: 'Logs' }).first()
    await expect(logsButton).toBeVisible()
    await logsButton.click()

    // Check for the sheet/panel with dialog role
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 })

    // Check for the log panel title containing "Logs"
    await expect(page.getByRole('heading', { name: /Logs/ })).toBeVisible()
  })

  test('should close log sheet when clicking outside or close', async ({ page }) => {
    // Open logs first
    const logsButton = page.getByRole('button', { name: 'Logs' }).first()
    await logsButton.click()

    // Wait for sheet to open
    const sheet = page.getByRole('dialog')
    await expect(sheet).toBeVisible()

    // Press Escape to close (more reliable than finding close button)
    await page.keyboard.press('Escape')

    // Sheet should be hidden
    await expect(sheet).toBeHidden({ timeout: 5000 })
  })

  test('should have Copy URL button on service cards', async ({ page }) => {
    // Check for Copy URL button
    const copyButton = page.getByRole('button', { name: 'Copy URL' }).first()
    await expect(copyButton).toBeVisible()
  })

  test('should navigate between tabs', async ({ page }) => {
    // Click Workflows tab
    const workflowsTab = page.getByRole('tab', { name: 'Flows' })
    await workflowsTab.click()

    // Check for workflows content
    await expect(page.getByText('Coming Soon').first()).toBeVisible()

    // Click Settings tab
    const settingsTab = page.getByRole('tab', { name: 'Settings' })
    await settingsTab.click()

    // Check for settings content (Coming Soon text)
    await expect(page.getByText('Settings - Coming Soon')).toBeVisible()

    // Click Docker tab to go back
    const dockerTab = page.getByRole('tab', { name: 'Docker' })
    await dockerTab.click()

    // Check for docker content
    await expect(page.locator('h2', { hasText: 'Dockers' })).toBeVisible()
  })

  test('should have Add DB button on database services', async ({ page }) => {
    // PostgreSQL is running, should have Add DB button
    const addDbButton = page.getByRole('button', { name: 'Add DB' }).first()
    await expect(addDbButton).toBeVisible()
  })

  test('should have Add vhost button on RabbitMQ', async ({ page }) => {
    // RabbitMQ should have Add vhost button (but disabled when stopped)
    const addVhostButton = page.getByRole('button', { name: 'Add vhost' })
    await expect(addVhostButton).toBeVisible()
  })

  test('should open Add DB dialog when clicking Add DB', async ({ page }) => {
    // First verify PostgreSQL is running and Add DB button is enabled
    const addDbButton = page.getByRole('button', { name: 'Add DB' }).first()
    await expect(addDbButton).toBeVisible()
    await expect(addDbButton).toBeEnabled()

    // Click Add DB button on PostgreSQL (which is running)
    await addDbButton.click()

    // Wait a bit for dialog animation
    await page.waitForTimeout(500)

    // Dialog should open
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 })
    await expect(page.getByRole('heading', { name: 'Create Database' })).toBeVisible()

    // Close dialog
    await page.keyboard.press('Escape')
    await expect(page.getByRole('dialog')).toBeHidden({ timeout: 5000 })
  })
})
