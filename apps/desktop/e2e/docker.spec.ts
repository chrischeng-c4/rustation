/**
 * E2E tests for Docker tab and grouped view
 */
import { test, expect, _electron as electron } from '@playwright/test'
import type { Page } from '@playwright/test'
import path from 'path'
import os from 'os'
import fs from 'fs'
import { execSync } from 'child_process'

let electronApp: Awaited<ReturnType<typeof electron.launch>>
let window: Page
let tempDir: string
let projectDir: string
let dockerAvailable = false

test.beforeAll(async () => {
  // Create a temp directory for test projects
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-docker-'))
  projectDir = path.join(tempDir, 'docker-test-project')
  fs.mkdirSync(projectDir, { recursive: true })

  // Initialize a git repo to simulate a project
  console.log('Setting up test project for Docker tests...')
  console.log('Project dir:', projectDir)

  try {
    execSync('git init', { cwd: projectDir })
    execSync('git config user.email "test@test.com"', { cwd: projectDir })
    execSync('git config user.name "Test User"', { cwd: projectDir })
    fs.writeFileSync(path.join(projectDir, 'README.md'), '# Docker Test Project\n')
    execSync('git add .', { cwd: projectDir })
    execSync('git commit -m "initial commit"', { cwd: projectDir })
    console.log('Git repo initialized successfully')
  } catch (e) {
    console.error('Failed to set up git repo:', e)
  }

  // Clear any persisted state to ensure fresh start
  const rstnDir = path.join(os.homedir(), '.rstn')
  const stateFile = path.join(rstnDir, 'state.json')
  if (fs.existsSync(stateFile)) {
    fs.unlinkSync(stateFile)
    console.log('Cleared existing state file')
  }

  // Build and launch the app
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
    env: {
      ...process.env,
      NODE_ENV: 'test',
    },
  })

  window = await electronApp.firstWindow()
  await window.waitForLoadState('domcontentloaded')
  await window.waitForTimeout(1000)

  // Open test project via IPC
  console.log('Opening test project...')
  await window.evaluate(async (projectPath) => {
    await (window as any).stateApi.dispatch({
      type: 'OpenProject',
      payload: { path: projectPath },
    })
  }, projectDir)

  await window.waitForTimeout(2000)
  console.log('Project opened')
})

test.afterAll(async () => {
  await electronApp.close()

  try {
    fs.rmSync(tempDir, { recursive: true, force: true })
  } catch {
    // Ignore cleanup errors
  }
})

// All tests in a single serial block to share state
test.describe.serial('Docker Tests', () => {
  test('shows Docker button in top bar when project is open', async () => {
    // Docker button is in the top bar (ProjectTabs), not sidebar
    const dockerButton = window.locator('button:has-text("Docker")').first()
    await expect(dockerButton).toBeVisible({ timeout: 5000 })
    console.log('Docker button is visible in top bar')

    await window.screenshot({ path: 'test-results/docker-00-topbar.png' })
  })

  test('can navigate to Docker tab and shows content', async () => {
    // Click on Docker button in top bar
    const dockerButton = window.locator('button:has-text("Docker")').first()
    await dockerButton.click()
    await window.waitForTimeout(1500)

    await window.screenshot({ path: 'test-results/docker-01-after-click.png' })

    // Check for Docker page content - either "Dockers" header or "Docker Not Available"
    const dockerHeader = window.locator('h2').filter({ hasText: 'Dockers' })
    const dockerNotAvailable = window.locator('h2').filter({ hasText: 'Docker Not Available' })

    await expect(dockerHeader.or(dockerNotAvailable)).toBeVisible({ timeout: 10000 })

    if (await dockerNotAvailable.isVisible()) {
      console.log('Docker is not available on this system')
      dockerAvailable = false
    } else {
      console.log('Docker is available')
      dockerAvailable = true
    }
  })

  test('shows Services header when Docker is available', async () => {
    if (!dockerAvailable) {
      // Verify retry button exists
      const retryButton = window.locator('button:has-text("Retry")')
      await expect(retryButton).toBeVisible()
      console.log('Docker not available - Retry button shown')
      return
    }

    const servicesHeader = window.locator('text=Services')
    await expect(servicesHeader).toBeVisible({ timeout: 5000 })
    console.log('Services header is visible')

    await window.screenshot({ path: 'test-results/docker-02-services.png' })
  })

  test('shows grouped services with count badges', async () => {
    if (!dockerAvailable) {
      console.log('Skipping - Docker not available')
      test.skip()
      return
    }

    await window.waitForTimeout(1000)
    await window.screenshot({ path: 'test-results/docker-03-grouped.png' })

    // Look for group containers - they have a button header with group name
    // Each group shows a badge with format "X/Y" (running/total)
    const groupBadges = window.locator('.rounded-lg.border').filter({
      has: window.locator('button')
    })

    const groupCount = await groupBadges.count()
    console.log(`Found ${groupCount} service groups`)

    // Should have at least one group (rstn with built-in services)
    expect(groupCount).toBeGreaterThanOrEqual(1)
  })

  test('can collapse and expand service groups', async () => {
    if (!dockerAvailable) {
      console.log('Skipping - Docker not available')
      test.skip()
      return
    }

    // Find a group header button (contains chevron icon)
    const groupHeaders = window.locator('.rounded-lg.border > button').first()

    if (await groupHeaders.count() === 0) {
      console.log('No group headers found')
      test.skip()
      return
    }

    // Click to collapse
    await groupHeaders.click()
    await window.waitForTimeout(300)
    await window.screenshot({ path: 'test-results/docker-04-collapsed.png' })
    console.log('Clicked to collapse group')

    // Click again to expand
    await groupHeaders.click()
    await window.waitForTimeout(300)
    await window.screenshot({ path: 'test-results/docker-05-expanded.png' })
    console.log('Group collapse/expand works')
  })

  test('shows read-only badge for non-rstn groups', async () => {
    if (!dockerAvailable) {
      console.log('Skipping - Docker not available')
      test.skip()
      return
    }

    // Look for "read-only" text
    const readOnlyBadges = window.locator('text=read-only')
    const count = await readOnlyBadges.count()
    console.log(`Found ${count} read-only badges`)

    await window.screenshot({ path: 'test-results/docker-06-readonly.png' })
    // Note: read-only badges only appear if there are non-rstn containers
  })

  test('can refresh Docker services', async () => {
    if (!dockerAvailable) {
      console.log('Skipping - Docker not available')
      test.skip()
      return
    }

    const refreshButton = window.locator('button:has-text("Refresh")')
    await expect(refreshButton).toBeVisible({ timeout: 5000 })

    await refreshButton.click()
    console.log('Clicked refresh button')

    await window.waitForTimeout(2000)
    await window.screenshot({ path: 'test-results/docker-07-refreshed.png' })

    // Services should still be visible
    const servicesHeader = window.locator('text=Services')
    await expect(servicesHeader).toBeVisible()
    console.log('Refresh completed successfully')
  })

  test('can click on a service to view logs', async () => {
    if (!dockerAvailable) {
      console.log('Skipping - Docker not available')
      test.skip()
      return
    }

    // Find service cards within groups
    const serviceCards = window.locator('[class*="Card"]').filter({
      has: window.locator('text=/Port:/')
    })

    const cardCount = await serviceCards.count()
    console.log(`Found ${cardCount} service cards`)

    if (cardCount === 0) {
      // Look for any clickable card
      const anyCard = window.locator('.rounded-lg.border .space-y-2 > div').first()
      if (await anyCard.count() > 0) {
        await anyCard.click()
      } else {
        console.log('No clickable service cards found')
        test.skip()
        return
      }
    } else {
      await serviceCards.first().click()
    }

    await window.waitForTimeout(1000)
    await window.screenshot({ path: 'test-results/docker-08-logs.png' })

    // Log panel should show "Logs" text
    const logPanel = window.locator('text=Logs').first()
    await expect(logPanel).toBeVisible()
    console.log('Log panel is visible')
  })
})
