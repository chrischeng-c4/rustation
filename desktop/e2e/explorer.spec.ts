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
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-explorer-'))
  projectDir = path.join(tempDir, 'test-project')
  fs.mkdirSync(projectDir, { recursive: true })

  // Initialize a git repo with some files
  try {
    execSync('git init', { cwd: projectDir, stdio: 'ignore' })
    execSync('git config user.email "test@test.com"', { cwd: projectDir, stdio: 'ignore' })
    execSync('git config user.name "Test"', { cwd: projectDir, stdio: 'ignore' })

    // Create test directory structure
    fs.mkdirSync(path.join(projectDir, 'src'), { recursive: true })
    fs.mkdirSync(path.join(projectDir, 'docs'), { recursive: true })

    // Create test files
    fs.writeFileSync(path.join(projectDir, 'README.md'), '# Test Project\n')
    fs.writeFileSync(path.join(projectDir, 'src', 'index.ts'), 'console.log("Hello")\n')
    fs.writeFileSync(path.join(projectDir, 'src', 'utils.ts'), 'export const add = (a, b) => a + b\n')
    fs.writeFileSync(path.join(projectDir, 'docs', 'guide.md'), '# Guide\n')

    execSync('git add . && git commit -m "init"', { cwd: projectDir, stdio: 'ignore' })

    // Create a modified file (not staged)
    fs.writeFileSync(path.join(projectDir, 'src', 'index.ts'), 'console.log("Hello World")\n')
  } catch {
    // Git initialization might fail in some environments
  }

  // Build and launch the app
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
    env: {
      ...process.env,
      RSTN_DATA_DIR: tempDir,
    },
  })
})

test.afterAll(async () => {
  if (electronApp) {
    await electronApp.close()
  }

  try {
    if (tempDir) {
      fs.rmSync(tempDir, { recursive: true, force: true })
    }
  } catch {
    // Ignore cleanup errors
  }
})

test.describe('File Explorer Tab', () => {
  test.skip('shows Explorer tab in sidebar', async () => {
    // This test is skipped because it requires opening a project
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    const explorerTab = window.locator('text=Explorer')
    await expect(explorerTab).toBeVisible()
  })

  test.skip('shows empty state when no worktree selected', async () => {
    // Skipped - requires navigation to Explorer tab
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Would need to:
    // 1. Navigate to Explorer tab
    // 2. Verify "No Worktree Selected" message
  })

  test.skip('displays file list when project is open', async () => {
    // Skipped - requires project to be opened
    const window = await electronApp.firstWindow()
    await window.waitForLoadState('domcontentloaded')

    // Would need to:
    // 1. Open project
    // 2. Navigate to Explorer tab
    // 3. Verify files are listed (README.md, src/, docs/)
  })
})

test.describe('File Navigation', () => {
  test.skip('can navigate into directories', async () => {
    // Skipped - requires project to be opened
    // Would test double-clicking on a directory to navigate into it
  })

  test.skip('can navigate back and forward', async () => {
    // Skipped - requires project to be opened
    // Would test using back/forward navigation buttons
  })

  test.skip('can navigate up to parent directory', async () => {
    // Skipped - requires project to be opened
    // Would test using the "up" button in navigation
  })

  test.skip('shows breadcrumb navigation', async () => {
    // Skipped - requires project to be opened
    // Would test clicking on breadcrumb segments to jump to parent paths
  })
})

test.describe('File Information', () => {
  test.skip('displays file metadata', async () => {
    // Skipped - requires project to be opened
    // Would test:
    // 1. Select a file
    // 2. Verify detail panel shows: size, modified date, git status
  })

  test.skip('shows git status indicators', async () => {
    // Skipped - requires project to be opened
    // Would test:
    // 1. Modified files show yellow indicator
    // 2. Clean files have no indicator
    // 3. Untracked files show blue indicator
  })

  test.skip('displays file icons correctly', async () => {
    // Skipped - requires project to be opened
    // Would test:
    // 1. Folders show folder icon
    // 2. Files show file icon
  })
})

test.describe('File Selection', () => {
  test.skip('can select files by clicking', async () => {
    // Skipped - requires project to be opened
    // Would test clicking on a file highlights it
  })

  test.skip('shows selected file in detail panel', async () => {
    // Skipped - requires project to be opened
    // Would test detail panel updates when file is selected
  })
})

// NOTE: Directory expansion E2E tests have been moved to explorer-expansion.spec.ts
// See that file for:
// - Unit test coverage documentation (passing test)
// - Future E2E test plans when infrastructure is ready
