import { Page, expect } from '@playwright/test'
import * as os from 'os'
import * as path from 'path'
import * as fs from 'fs/promises'

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
 * Create a temporary test project directory
 */
export async function createTestProject(): Promise<string> {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'rstn-test-'))

  // Initialize as git repo (rstn requires git)
  await fs.mkdir(path.join(tmpDir, '.git'))

  // Create a minimal justfile
  await fs.writeFile(path.join(tmpDir, 'justfile'), 'test:\n\techo "test"')

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
