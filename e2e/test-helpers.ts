import { Page, expect } from '@playwright/test'
import * as os from 'os'
import * as path from 'path'
import * as fs from 'fs/promises'
import { exec } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)

/**
 * Open a project programmatically via state dispatch
 */
export async function openProject(page: Page, projectPath: string): Promise<void> {
  await page.evaluate(
    async (path: string) => {
      await (window as any).stateApi.dispatch({
        type: 'OpenProject',
        payload: { path }
      })
    },
    projectPath
  )

  // Poll state until project is loaded (worktree exists)
  await page.waitForFunction(
    async () => {
      const json = await (window as any).stateApi.getState()
      const state = JSON.parse(json)
      // Note: active_project is not serialized, use projects[active_project_index]
      const activeProject = state?.projects?.[state?.active_project_index]
      return (
        activeProject?.worktrees &&
        activeProject.worktrees.length > 0
      )
    },
    { timeout: 10000 }
  )

  // Wait for UI to actually show the project is loaded (Commands panel appears)
  await expect(page.getByText('Commands', { exact: true })).toBeVisible({ timeout: 5000 })

  // CRITICAL: Wait for project to stabilize - give async operations time to complete
  // Without this, the project may close immediately after loading
  await page.waitForTimeout(2000)

  // Verify project is still loaded after stabilization
  const stateAfterWait = await page.evaluate(async () => {
    const json = await (window as any).stateApi.getState()
    return JSON.parse(json)
  })

  // Note: active_project is not serialized, use projects[active_project_index]
  const activeProject = stateAfterWait?.projects?.[stateAfterWait?.active_project_index]
  if (!activeProject?.worktrees || activeProject.worktrees.length === 0) {
    throw new Error('Project closed after initial load - state validation failed')
  }
}

/**
 * Create a temporary test project directory with valid git repo
 */
export async function createTestProject(): Promise<string> {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'rstn-test-'))

  // Initialize as proper git repo (rstn requires valid git)
  await execAsync('git init', { cwd: tmpDir })
  await execAsync('git config user.name "Test User"', { cwd: tmpDir })
  await execAsync('git config user.email "test@example.com"', { cwd: tmpDir })

  // Create a minimal justfile with test commands
  const justfileContent = `# Test project justfile
test:
\techo "Running tests"

build:
\techo "Building project"

constitution-init:
\techo "Initialize constitution"
`
  await fs.writeFile(path.join(tmpDir, 'justfile'), justfileContent)

  // Create initial commit so git repo is fully valid
  await execAsync('git add .', { cwd: tmpDir })
  await execAsync('git commit -m "Initial commit"', { cwd: tmpDir })

  return tmpDir
}

/**
 * Cleanup test project
 */
export async function cleanupTestProject(projectPath: string): Promise<void> {
  await fs.rm(projectPath, { recursive: true, force: true })
}

/**
 * Capture console errors from page
 */
export function captureConsoleErrors(page: Page): string[] {
  const errors: string[] = []

  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(msg.text())
    }
  })

  page.on('pageerror', (error) => {
    errors.push(error.message)
  })

  return errors
}

/**
 * Get current state from the app
 */
export async function getAppState(page: Page): Promise<any> {
  return await page.evaluate(async () => {
    const json = await (window as any).stateApi.getState()
    return JSON.parse(json)
  })
}
