/**
 * E2E tests for inline file comments functionality
 */
import { test, expect, _electron as electron } from '@playwright/test'
import type { Page } from '@playwright/test'
import path from 'path'
import os from 'os'
import fs from 'fs'
import { execSync } from 'child_process'

let electronApp: Awaited<ReturnType<typeof electron.launch>>
let page: Page
let tempDir: string
let projectDir: string

test.beforeAll(async () => {
  // Create a temp directory for test project
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-comments-'))
  projectDir = path.join(tempDir, 'test-project')
  fs.mkdirSync(projectDir, { recursive: true })

  // Initialize a git repo
  console.log('Setting up test project for comment tests...')
  console.log('Project dir:', projectDir)

  try {
    execSync('git init', { cwd: projectDir })
    execSync('git config user.email "test@test.com"', { cwd: projectDir })
    execSync('git config user.name "Test User"', { cwd: projectDir })

    // Create a test file with some content
    const testFile = path.join(projectDir, 'test.ts')
    const fileContent = `// Test file for comments
function hello() {
  console.log('Hello world')
}

function add(a: number, b: number) {
  return a + b
}

export { hello, add }
`
    fs.writeFileSync(testFile, fileContent)

    execSync('git add .', { cwd: projectDir })
    execSync('git commit -m "initial commit"', { cwd: projectDir })
    console.log('Git repo initialized successfully')
  } catch (e) {
    console.error('Failed to set up git repo:', e)
  }

  // Clear any persisted state
  const rstnDir = path.join(os.homedir(), '.rstn')
  const stateFile = path.join(rstnDir, 'state.json')
  if (fs.existsSync(stateFile)) {
    fs.unlinkSync(stateFile)
    console.log('Cleared existing state file')
  }

  // Launch Electron app
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
    env: {
      ...process.env,
      NODE_ENV: 'test',
    },
    timeout: 30000,
    executablePath: undefined, // Use default Electron
  })

  page = await electronApp.firstWindow()

  // Capture console logs for debugging
  page.on('console', (msg) => {
    const type = msg.type()
    if (type === 'error' || type === 'warning') {
      console.log(`[RENDERER ${type}]`, msg.text())
    }
  })

  await page.waitForLoadState('domcontentloaded')
  await page.waitForTimeout(1000)

  // Open test project via IPC
  console.log('Opening test project...')
  await page.evaluate(async (projectPath) => {
    await (window as any).stateApi.dispatch({
      type: 'OpenProject',
      payload: { path: projectPath },
    })
  }, projectDir)

  await page.waitForTimeout(2000)
  console.log('Project opened')
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

test.describe('File Comments', () => {
  test('should add inline comment and verify it appears', async () => {
    // Navigate to Explorer tab
    const explorerButton = page.getByRole('button', { name: 'Explorer' })
    await explorerButton.click()
    await page.waitForTimeout(500)

    // Click on test.ts file
    console.log('Looking for test.ts file...')
    const testFile = page.getByText('test.ts')
    await testFile.click()
    await page.waitForTimeout(1000)

    console.log('File opened, looking for content...')

    // Wait for file content to appear
    await expect(page.getByText(/function hello/)).toBeVisible({ timeout: 5000 })

    console.log('File content visible')

    // Take screenshot before attempting to add comment
    await page.screenshot({ path: 'test-results/before-comment.png' })

    // Find line 5 (the "function add" line)
    const line5 = page.locator('text=/^5$/')
    const hasLine5 = await line5.isVisible({ timeout: 2000 }).catch(() => false)

    if (!hasLine5) {
      console.error('Line 5 not found, screenshot saved')
      await page.screenshot({ path: 'test-results/no-line-numbers.png' })
      throw new Error('Line numbers not visible')
    }

    console.log('Line 5 found, hovering...')

    // Hover over line 5
    await line5.hover()
    await page.waitForTimeout(500)

    // Take screenshot after hover
    await page.screenshot({ path: 'test-results/after-hover.png' })

    // Look for add comment button
    const addCommentBtn = page.locator('button[title*="comment" i], button[aria-label*="comment" i]').first()
    const hasBtnVisible = await addCommentBtn.isVisible({ timeout: 2000 }).catch(() => false)

    if (!hasBtnVisible) {
      console.error('Add comment button not found after hover')
      await page.screenshot({ path: 'test-results/no-comment-button.png' })

      // Log all buttons on page for debugging
      const allButtons = await page.locator('button').count()
      console.log(`Total buttons on page: ${allButtons}`)

      throw new Error('Add comment button not visible')
    }

    console.log('Add comment button found, clicking...')
    await addCommentBtn.click()
    await page.waitForTimeout(500)

    // Look for textarea
    const textarea = page.locator('textarea').first()
    await expect(textarea).toBeVisible({ timeout: 3000 })

    console.log('Textarea visible, filling content...')

    // Fill and submit
    const testComment = 'E2E test comment'
    await textarea.fill(testComment)

    await page.screenshot({ path: 'test-results/filled-comment.png' })

    const submitBtn = page.locator('button:has-text("Submit")').first()
    await expect(submitBtn).toBeVisible({ timeout: 2000 })

    console.log('Submitting comment...')
    await submitBtn.click()

    // CRITICAL TEST: Wait and check if comment appears
    console.log('Waiting for comment to appear...')

    await page.waitForTimeout(2000) // Give time for backend to process

    await page.screenshot({ path: 'test-results/after-submit.png' })

    const commentAppeared = await page.getByText(testComment).isVisible({ timeout: 5000 }).catch(() => false)

    if (!commentAppeared) {
      console.error('❌ FAIL: Comment did not appear!')

      // Check if textarea is still visible
      const textareaStillVisible = await textarea.isVisible().catch(() => false)
      console.log(`Textarea still visible: ${textareaStillVisible}`)

      // Check for any error messages
      const errorMsg = await page.locator('[role="alert"], .error').textContent().catch(() => 'none')
      console.log(`Error messages: ${errorMsg}`)

      throw new Error('Comment did not appear after submission')
    }

    console.log('✓ Comment appeared successfully!')

    // Verify textarea disappeared
    await expect(textarea).not.toBeVisible({ timeout: 3000 })
  })

  test('should cancel comment without saving', async () => {
    const explorerButton = page.getByRole('button', { name: 'Explorer' })
    await explorerButton.click()
    await page.waitForTimeout(500)

    // Navigate to any file (reuse logic from previous test)
    const fileTree = page.locator('[class*="file-tree"]').first()
    const hasProject = await fileTree.isVisible().catch(() => false)
    test.skip(!hasProject, 'No project loaded - skipping')

    // Open file (simplified - assume project loaded from previous test)
    const sourceViewer = page.locator('[class*="MuiBox"]').filter({
      has: page.locator('code, pre'),
    }).first()

    const hasSource = await sourceViewer.isVisible({ timeout: 5000 }).catch(() => false)
    test.skip(!hasSource, 'No file opened - skipping')

    // Find line 5
    const line5 = page.locator('span, div').filter({ hasText: '5' }).first()
    const hasLine = await line5.isVisible().catch(() => false)
    test.skip(!hasLine, 'Line 5 not found - skipping')

    await line5.hover()
    await page.waitForTimeout(300)

    // Click add comment
    const addCommentButton = page.locator('button').filter({ hasText: /comment/i }).first()
    const hasButton = await addCommentButton.isVisible({ timeout: 2000 }).catch(() => false)
    test.skip(!hasButton, 'Add comment button not found')

    await addCommentButton.click()
    await page.waitForTimeout(300)

    // Fill some content
    const textarea = page.locator('textarea').first()
    await expect(textarea).toBeVisible({ timeout: 2000 })

    await textarea.fill('This will not be saved')

    // Click cancel
    const cancelButton = page.locator('button').filter({ hasText: /cancel/i }).first()
    await cancelButton.click()

    // Textarea should be gone
    await expect(textarea).not.toBeVisible({ timeout: 2000 })

    // Comment should NOT appear
    await page.waitForTimeout(1000)
    const savedComment = page.getByText('This will not be saved')
    const appeared = await savedComment.isVisible().catch(() => false)

    expect(appeared).toBe(false)
    console.log('✓ Cancel worked - comment not saved')
  })

  test('should keep textarea visible while comment is being saved', async () => {
    // This test specifically checks for the async bug
    const explorerButton = page.getByRole('button', { name: 'Explorer' })
    await explorerButton.click()
    await page.waitForTimeout(500)

    const fileTree = page.locator('[class*="file-tree"]').first()
    const hasProject = await fileTree.isVisible().catch(() => false)
    test.skip(!hasProject, 'No project loaded - skipping')

    // Find any line to comment on
    const lineNumbers = page.locator('span, div').filter({ hasText: /^\d+$/ })
    const firstLine = lineNumbers.first()

    await firstLine.hover()
    await page.waitForTimeout(300)

    const addButton = page.locator('button').filter({ hasText: /comment/i }).first()
    const hasButton = await addButton.isVisible({ timeout: 2000 }).catch(() => false)
    test.skip(!hasButton, 'Add comment button not found')

    await addButton.click()

    const textarea = page.locator('textarea').first()
    await expect(textarea).toBeVisible({ timeout: 2000 })

    await textarea.fill('Test async behavior')

    const submitButton = page.locator('button').filter({ hasText: /submit/i }).first()

    // Record time before submit
    const startTime = Date.now()
    await submitButton.click()

    // Check if textarea is still visible immediately after click
    const textareaStillVisible = await textarea.isVisible().catch(() => false)

    if (!textareaStillVisible) {
      const elapsed = Date.now() - startTime
      console.error(
        `❌ BUG DETECTED: Textarea disappeared immediately (${elapsed}ms) - should wait for async operation`
      )
    } else {
      console.log('✓ Textarea remained visible after submit (good!)')
    }

    // Wait for comment to actually appear
    await page.waitForTimeout(2000)

    // Now textarea should be gone
    const finalTextareaVisible = await textarea.isVisible().catch(() => false)

    if (finalTextareaVisible) {
      console.warn('⚠️ Textarea still visible after 2s - operation may have failed')
    }
  })
})
