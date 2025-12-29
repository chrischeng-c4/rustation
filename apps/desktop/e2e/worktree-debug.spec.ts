/**
 * Debug test for worktree tabs visibility
 * Takes screenshots to investigate worktree tabs behavior
 */
import { test, expect, _electron as electron } from '@playwright/test'
import path from 'path'
import os from 'os'
import fs from 'fs'
import { execSync } from 'child_process'

let electronApp: Awaited<ReturnType<typeof electron.launch>>
let tempDir: string
let projectDir: string

test.beforeAll(async () => {
  // Create a temp directory for test projects
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-wt-debug-'))
  projectDir = path.join(tempDir, 'single-worktree-project')
  fs.mkdirSync(projectDir, { recursive: true })

  // Initialize a git repo with SINGLE worktree (to test the fix)
  console.log('Setting up test project with SINGLE worktree...')
  console.log('Project dir:', projectDir)

  try {
    // Initialize main repo
    execSync('git init', { cwd: projectDir })
    execSync('git config user.email "test@test.com"', { cwd: projectDir })
    execSync('git config user.name "Test User"', { cwd: projectDir })
    fs.writeFileSync(path.join(projectDir, 'README.md'), '# Test Project\n')
    execSync('git add .', { cwd: projectDir })
    execSync('git commit -m "initial commit"', { cwd: projectDir })

    // Verify worktrees (should only have 1)
    const worktreeList = execSync('git worktree list', { cwd: projectDir, encoding: 'utf-8' })
    console.log('Git worktree list output:')
    console.log(worktreeList)
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
})

test.afterAll(async () => {
  await electronApp.close()

  try {
    fs.rmSync(tempDir, { recursive: true, force: true })
  } catch {
    // Ignore cleanup errors
  }
})

test.describe('Worktree Debug', () => {
  test('worktree row shows even with single worktree', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Screenshot 1: Initial state (should show No Project Open)
    await window.screenshot({ path: 'test-results/01-initial-state.png' })
    console.log('Screenshot 1: Initial state saved')

    // Wait a bit for state to settle
    await window.waitForTimeout(1000)

    // Open the test project via IPC (simulating dispatch)
    await window.evaluate(async (projectPath) => {
      await (window as any).stateApi.dispatch({
        type: 'OpenProject',
        payload: { path: projectPath },
      })
    }, projectDir)

    // Wait for project to load
    await window.waitForTimeout(2000)

    // Screenshot 2: After opening project
    await window.screenshot({ path: 'test-results/02-after-open-project.png' })
    console.log('Screenshot 2: After opening project saved')

    // Get the current state to debug
    const stateJson = await window.evaluate(async () => {
      return await (window as any).stateApi.getState()
    })
    const state = JSON.parse(stateJson)

    console.log('=== DEBUG STATE ===')
    console.log('Projects count:', state.projects?.length)
    if (state.projects?.[0]) {
      const project = state.projects[0]
      console.log('Project path:', project.path)
      console.log('Project name:', project.name)
      console.log('Worktrees count:', project.worktrees?.length)
      if (project.worktrees?.[0]) {
        console.log('First worktree branch:', project.worktrees[0].branch)
      }
    }
    console.log('===================')

    // Check worktrees in state
    const worktreesCount = state.projects?.[0]?.worktrees?.length ?? 0
    console.log('Worktrees in state:', worktreesCount)

    // Assertions - should have exactly 1 worktree
    expect(state.projects?.length).toBe(1)
    expect(worktreesCount).toBe(1)

    // NEW: Worktree row should now be visible even with 1 worktree
    console.log('Checking if worktree row is visible with single worktree...')

    // Look for the worktree row (has border-t class)
    const worktreeRow = window.locator('.border-t.border-border\\/50.bg-muted\\/20')
    await expect(worktreeRow).toBeVisible({ timeout: 5000 })
    console.log('Worktree row is visible!')

    // Check for the branch name in the worktree tab
    const branchTab = window.locator('text=main').first()
    await expect(branchTab).toBeVisible()
    console.log('Branch tab "main" is visible!')

    // Check for the "+" button to add worktree (small icon button with h-6 w-6 classes)
    const addWorktreeButton = worktreeRow.locator('button.h-6.w-6').first()
    await expect(addWorktreeButton).toBeVisible()
    console.log('Add worktree button is visible!')

    // Final screenshot
    await window.screenshot({ path: 'test-results/03-single-worktree-with-tabs.png' })
    console.log('Screenshot 3: Single worktree with tabs visible saved')
  })

  test('add worktree dialog opens and shows branches', async () => {
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Wait for any state to settle
    await window.waitForTimeout(500)

    // Find the worktree row and the "+" button (small icon button with h-6 w-6 classes)
    const worktreeRow = window.locator('.border-t.border-border\\/50.bg-muted\\/20')
    const addWorktreeButton = worktreeRow.locator('button.h-6.w-6').first()

    // Click the add worktree button
    await addWorktreeButton.click()
    console.log('Clicked add worktree button')

    // Wait for dialog to open
    await window.waitForTimeout(500)

    // Screenshot: Dialog opened
    await window.screenshot({ path: 'test-results/04-add-worktree-dialog.png' })
    console.log('Screenshot 4: Add worktree dialog saved')

    // Check that dialog is visible (use heading role to be specific)
    const dialogTitle = window.getByRole('heading', { name: 'Add Worktree' })
    await expect(dialogTitle).toBeVisible({ timeout: 5000 })
    console.log('Dialog title is visible!')

    // Check for "Available Branches" label
    const branchesLabel = window.locator('text=Available Branches')
    await expect(branchesLabel).toBeVisible()
    console.log('Available Branches label is visible!')

    // Check for "Create New Branch" button
    const createNewBranchButton = window.locator('button:has-text("Create New Branch")')
    await expect(createNewBranchButton).toBeVisible()
    console.log('Create New Branch button is visible!')

    // Final screenshot with dialog
    await window.screenshot({ path: 'test-results/05-dialog-with-branches.png' })
    console.log('Screenshot 5: Dialog with branches saved')

    // Close the dialog by clicking Cancel
    const cancelButton = window.locator('button:has-text("Cancel")')
    await cancelButton.click()
    await window.waitForTimeout(300)

    // Verify dialog closed
    await expect(dialogTitle).not.toBeVisible({ timeout: 3000 })
    console.log('Dialog closed successfully!')
  })
})
