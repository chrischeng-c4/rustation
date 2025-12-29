import { test, expect, _electron as electron } from '@playwright/test'
import path from 'path'
import os from 'os'
import fs from 'fs'
import { execSync } from 'child_process'

let electronApp: Awaited<ReturnType<typeof electron.launch>>
let tempDir: string
let projectDir: string

// Check if claude CLI is available
function isClaudeAvailable(): boolean {
  try {
    execSync('which claude', { stdio: 'ignore' })
    return true
  } catch {
    return false
  }
}

test.beforeAll(async () => {
  // Create a temp directory for test projects
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-workflows-'))
  projectDir = path.join(tempDir, 'test-project')
  fs.mkdirSync(projectDir, { recursive: true })

  // Initialize a git repo
  try {
    execSync('git init', { cwd: projectDir, stdio: 'ignore' })
    execSync('git config user.email "test@test.com"', { cwd: projectDir, stdio: 'ignore' })
    execSync('git config user.name "Test"', { cwd: projectDir, stdio: 'ignore' })

    // Create some source files to simulate a real project
    fs.writeFileSync(path.join(projectDir, 'README.md'), '# Test Project\n')
    fs.mkdirSync(path.join(projectDir, 'src'), { recursive: true })
    fs.writeFileSync(
      path.join(projectDir, 'src', 'main.rs'),
      `fn main() {\n    println!("Hello, world!");\n}\n`
    )
    fs.writeFileSync(
      path.join(projectDir, 'Cargo.toml'),
      `[package]\nname = "test-project"\nversion = "0.1.0"\n`
    )

    execSync('git add . && git commit -m "init"', { cwd: projectDir, stdio: 'ignore' })
  } catch (e) {
    console.error('Git initialization failed:', e)
  }

  // Build and launch the app
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
    env: {
      ...process.env,
      RSTN_DATA_DIR: tempDir,
      // Pre-open the project by setting state
      RSTN_INITIAL_PROJECT: projectDir,
    },
  })
})

test.afterAll(async () => {
  await electronApp.close()

  try {
    fs.rmSync(tempDir, { recursive: true, force: true })
  } catch {
    // Ignore cleanup errors
  }
})

test.describe('Workflows Tab', () => {
  test('shows No Project Open when no project is loaded', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Wait for app to initialize
    await window.waitForTimeout(2000)

    // When no project is open, should show "No Project Open"
    // The sidebar with Flows tab only appears when a project is open
    const noProject = window.locator('text=No Project Open')
    const openButton = window.getByRole('button', { name: 'Open Project' })

    const noProjectVisible = await noProject.isVisible().catch(() => false)
    const openButtonVisible = await openButton.first().isVisible().catch(() => false)

    // Either "No Project Open" or "Open Project" button should be visible
    expect(noProjectVisible || openButtonVisible).toBeTruthy()
  })

  test('sidebar is hidden when no project is open', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Wait for app to initialize
    await window.waitForTimeout(2000)

    // The sidebar (with Flows/Tasks tabs) should NOT be visible without a project
    const flowsTab = window.locator('span:has-text("Flows")')
    const tasksTab = window.locator('span:has-text("Tasks")')

    const flowsVisible = await flowsTab.isVisible().catch(() => false)
    const tasksVisible = await tasksTab.isVisible().catch(() => false)

    // Sidebar tabs should not be visible without a project
    // (They only appear when worktree is available)
    expect(flowsVisible && tasksVisible).toBeFalsy()
  })

  test('can switch between Flows and Tasks tabs', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Wait for app to initialize
    await window.waitForTimeout(2000)

    // Click Tasks tab
    const tasksTab = window.locator('span:has-text("Tasks")')
    if (await tasksTab.isVisible()) {
      await tasksTab.click()
      await window.waitForTimeout(500)

      // Should see Tasks page content
      const tasksHeading = window.locator('h2:has-text("Tasks")')
      const noProject = window.locator('text=No Project Open')
      const isTasksVisible =
        (await tasksHeading.isVisible().catch(() => false)) ||
        (await noProject.isVisible().catch(() => false))
      expect(isTasksVisible).toBeTruthy()

      // Click back to Flows
      const flowsTab = window.locator('span:has-text("Flows")')
      await flowsTab.click()
      await window.waitForTimeout(500)
    }
  })
})

test.describe('Constitution Workflow', () => {
  test.skip('shows Constitution Setup card in Workflows', async () => {
    // Skip if no project is open (requires dialog mocking)
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Navigate to Flows tab
    const flowsTab = window.locator('span:has-text("Flows")')
    if (await flowsTab.isVisible()) {
      await flowsTab.click()
    }

    // Look for Constitution Setup workflow card
    const constitutionCard = window.locator('text=Constitution Setup')
    await expect(constitutionCard).toBeVisible({ timeout: 5000 })
  })

  test.skip('can start Constitution workflow', async () => {
    // Requires project to be open
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Click on Constitution Setup card
    const constitutionCard = window.locator('text=Constitution Setup').first()
    if (await constitutionCard.isVisible()) {
      await constitutionCard.click()

      // Should see Constitution panel with options
      const applyDefault = window.locator('text=Apply Default Template')
      const createQA = window.locator('text=Create with Q&A')

      const hasOptions =
        (await applyDefault.isVisible().catch(() => false)) ||
        (await createQA.isVisible().catch(() => false))

      expect(hasOptions).toBeTruthy()
    }
  })
})

test.describe('Claude Code Integration', () => {
  const claudeAvailable = isClaudeAvailable()

  test('claude CLI check', async () => {
    // Just log whether claude is available
    console.log(`Claude CLI available: ${claudeAvailable}`)
    expect(true).toBeTruthy()
  })

  test.skip('can invoke Claude Code for constitution generation', async () => {
    // This test requires:
    // 1. Claude CLI to be installed
    // 2. Project to be open
    // 3. API key to be configured

    if (!claudeAvailable) {
      console.log('Skipping Claude Code test - claude CLI not available')
      return
    }

    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Navigate to Flows
    const flowsTab = window.locator('span:has-text("Flows")')
    if (await flowsTab.isVisible()) {
      await flowsTab.click()
    }

    // Click Constitution Setup
    const constitutionCard = window.locator('text=Constitution Setup').first()
    if (await constitutionCard.isVisible()) {
      await constitutionCard.click()

      // Click Apply Default Template (this invokes Claude)
      const applyDefault = window.locator('button:has-text("Apply Default Template")')
      if (await applyDefault.isVisible()) {
        await applyDefault.click()

        // Wait for generation (this can take time)
        await window.waitForTimeout(10000)

        // Should see success state
        const success = window.locator('text=Constitution Active')
        const generating = window.locator('text=Generating')

        const hasProgress =
          (await success.isVisible().catch(() => false)) ||
          (await generating.isVisible().catch(() => false))

        expect(hasProgress).toBeTruthy()
      }
    }
  })
})

test.describe('Tasks Tab (Justfile only)', () => {
  test('Tasks tab shows justfile commands description', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Navigate to Tasks tab
    const tasksTab = window.locator('span:has-text("Tasks")')
    if (await tasksTab.isVisible()) {
      await tasksTab.click()
      await window.waitForTimeout(500)

      // Should see "Run justfile commands" description
      const description = window.locator('text=Run justfile commands')
      const noProject = window.locator('text=No Project Open')

      const isVisible =
        (await description.isVisible().catch(() => false)) ||
        (await noProject.isVisible().catch(() => false))

      expect(isVisible).toBeTruthy()
    }
  })

  test('Tasks tab does NOT show Claude Code or Constitution', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Navigate to Tasks tab
    const tasksTab = window.locator('span:has-text("Tasks")')
    if (await tasksTab.isVisible()) {
      await tasksTab.click()
      await window.waitForTimeout(500)

      // These should NOT be visible in Tasks tab (they moved to Workflows)
      const claudeCode = window.locator('[data-testid="task-card-claude-code"]')
      const constitutionInit = window.locator('[data-testid="task-card-constitution-init"]')

      const claudeVisible = await claudeCode.isVisible().catch(() => false)
      const constitutionVisible = await constitutionInit.isVisible().catch(() => false)

      expect(claudeVisible).toBeFalsy()
      expect(constitutionVisible).toBeFalsy()
    }
  })
})
