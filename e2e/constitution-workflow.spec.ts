import { test, expect } from './electron.fixture'
import path from 'path'
import {
  openProject,
  createTestProject,
  cleanupTestProject,
  captureConsoleErrors,
  getAppState
} from './test-helpers'

test.describe('Constitution Workflow - Full Integration', () => {
  let testProjectPath: string
  let consoleErrors: string[]

  test.beforeEach(async ({ page }) => {
    // Use the actual rustation project instead of creating a fake one
    // This ensures we have a valid git repo with proper structure
    // Use dynamic path resolution to work on any developer's machine
    testProjectPath = path.resolve(__dirname, '..')
    await openProject(page, testProjectPath)

    // Setup console error capture
    consoleErrors = captureConsoleErrors(page)

    // Tasks tab is selected by default when project opens
    // No need to click it explicitly
  })

  test.afterEach(async () => {
    // No cleanup needed - we're using the actual rustation project
  })

  test('should display Initialize Constitution command', async ({ page }) => {
    // Verify command appears in list (searches for command name)
    const cmd = page.getByText('constitution-init')
    await expect(cmd).toBeVisible()

    // Should have description
    await expect(page.getByText('Initialize project constitution (CESDD)')).toBeVisible()

    // No console errors
    expect(consoleErrors).toHaveLength(0)
  })

  test('should show ConstitutionPanel when command is clicked', async ({ page }) => {
    // Debug: Check state BEFORE clicking
    let state = await getAppState(page)
    const activeProject = state?.projects?.[state?.active_project_index]
    console.log('BEFORE CLICK - Has project:', !!activeProject)
    console.log('BEFORE CLICK - Has worktrees:', activeProject?.worktrees?.length)

    // Click the play button for constitution-init command
    // Use data-testid for reliable selection
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')

    // Check if button exists before clicking
    const buttonCount = await playButton.count()
    console.log('Button count:', buttonCount)

    if (buttonCount > 0) {
      await playButton.click()
    } else {
      throw new Error('Constitution init button not found!')
    }

    // Wait longer for state to propagate and panel to render
    await page.waitForTimeout(3000)

    // Check state to debug
    state = await getAppState(page)
    const activeProjectAfter = state?.projects?.[state?.active_project_index]
    console.log('AFTER CLICK - Active command:', activeProjectAfter?.worktrees?.[0]?.tasks?.active_command)
    console.log('AFTER CLICK - Workflow:', activeProjectAfter?.worktrees?.[0]?.tasks?.constitution_workflow)

    // Panel should appear - use heading role to avoid matching both span and h3
    const question = page.getByRole('heading', { name: /What technology stack/i })
    await expect(question).toBeVisible({ timeout: 5000 })

    // Should show progress 0/4
    await expect(page.getByText('0 / 4')).toBeVisible()

    // Should have input field
    const textarea = page.getByPlaceholder('Type your answer...')
    await expect(textarea).toBeVisible()

    // Next button should be disabled
    const nextButton = page.getByRole('button', { name: /Next/i })
    await expect(nextButton).toBeDisabled()

    // No console errors (critical check!)
    expect(consoleErrors.filter((e) => !e.includes('Electron Security Warning'))).toHaveLength(0)
  })

  test('should enable Next button when answer is typed', async ({ page }) => {
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')
    await playButton.click()
    await page.waitForTimeout(500)

    const textarea = page.getByPlaceholder('Type your answer...')
    const nextButton = page.getByRole('button', { name: /Next/i })

    // Type answer
    await textarea.fill('React + Rust + TypeScript')

    // Button should be enabled
    await expect(nextButton).toBeEnabled()

    // Clear answer - button should disable
    await textarea.clear()
    await expect(nextButton).toBeDisabled()
  })

  test('should advance through all 4 questions', async ({ page }) => {
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')
    await playButton.click()
    await page.waitForTimeout(500)

    const textarea = page.getByPlaceholder('Type your answer...')
    const nextButton = page.getByRole('button', { name: /Next/i })

    // Question 1: Technology Stack
    await expect(page.getByText('0 / 4')).toBeVisible()
    await textarea.fill('React + Rust')
    await nextButton.click()
    await page.waitForTimeout(300)

    // Question 2: Security
    await expect(page.getByText('1 / 4')).toBeVisible()
    await expect(page.getByRole('heading', { name: /security requirements/i })).toBeVisible()
    await textarea.fill('JWT auth')
    await nextButton.click()
    await page.waitForTimeout(300)

    // Question 3: Code Quality
    await expect(page.getByText('2 / 4')).toBeVisible()
    await textarea.fill('80% coverage')
    await nextButton.click()
    await page.waitForTimeout(300)

    // Question 4: Architecture
    await expect(page.getByText('3 / 4')).toBeVisible()
    await textarea.fill('State-first')
    await nextButton.click()
    await page.waitForTimeout(500)

    // Should show "All questions answered"
    await expect(page.getByText(/All questions answered/i)).toBeVisible()

    // Should show Generate button
    const generateButton = page.getByRole('button', { name: /Generate Constitution/i })
    await expect(generateButton).toBeVisible()
    await expect(generateButton).toBeEnabled()

    // Verify state
    const state = await getAppState(page)
    const workflow = state.projects[0]?.worktrees[0]?.tasks?.constitution_workflow
    expect(workflow).toBeDefined()
    expect(workflow.current_question).toBe(4)
    expect(workflow.answers).toHaveProperty('tech_stack')
    expect(workflow.answers).toHaveProperty('security')
    expect(workflow.answers).toHaveProperty('code_quality')
    expect(workflow.answers).toHaveProperty('architecture')
  })

  test('should show checkmarks for answered questions', async ({ page }) => {
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')
    await playButton.click()
    await page.waitForTimeout(500)

    const textarea = page.getByPlaceholder('Type your answer...')
    const nextButton = page.getByRole('button', { name: /Next/i })

    // Answer 2 questions
    await textarea.fill('React')
    await nextButton.click()
    await page.waitForTimeout(300)

    await textarea.fill('JWT')
    await nextButton.click()
    await page.waitForTimeout(300)

    // Should see checkmarks (CheckCircle icons with green color)
    // The CheckCircle SVG elements have the text-green-500 class directly
    const checkmarks = page.locator('svg.text-green-500')
    const count = await checkmarks.count()
    expect(count).toBeGreaterThanOrEqual(2)
  })

  test('should preserve state when navigating away and back', async ({ page }) => {
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')
    await playButton.click()
    await page.waitForTimeout(500)

    // Answer 2 questions
    const textarea = page.getByPlaceholder('Type your answer...')
    const nextButton = page.getByRole('button', { name: /Next/i })

    await textarea.fill('React')
    await nextButton.click()
    await page.waitForTimeout(300)

    await textarea.fill('JWT')
    await nextButton.click()
    await page.waitForTimeout(300)

    // Currently on question 3 (2/4 progress)
    await expect(page.getByText('2 / 4')).toBeVisible()

    // Navigate to Settings (if available)
    const settingsButton = page.getByRole('button', { name: /Settings/i })
    const settingsVisible = await settingsButton.isVisible().catch(() => false)

    if (settingsVisible) {
      await settingsButton.click()
      await page.waitForTimeout(500)

      // Navigate back to Tasks
      const tasksButton = page.getByRole('button', { name: /Tasks/i })
      await tasksButton.click()
      await page.waitForTimeout(500)

      // Select Constitution again
      const playButtonAgain = page.getByTestId('task-card-constitution-init').locator('button')
      await playButtonAgain.click()
      await page.waitForTimeout(500)

      // Should still be on question 3
      await expect(page.getByText('2 / 4')).toBeVisible()
      await expect(page.getByRole('heading', { name: /code quality/i })).toBeVisible()
    }
  })

  test('should handle Generate Constitution click', async ({ page }) => {
    // Complete all 4 questions
    const playButton = page.getByTestId('task-card-constitution-init').locator('button')
    await playButton.click()
    await page.waitForTimeout(500)

    const textarea = page.getByPlaceholder('Type your answer...')
    const nextButton = page.getByRole('button', { name: /Next/i })

    const answers = ['React + Rust', 'JWT auth', '80% coverage', 'State-first']
    for (const answer of answers) {
      await textarea.fill(answer)
      await nextButton.click()
      await page.waitForTimeout(200)
    }

    // Verify "All questions answered" message appears
    await expect(page.getByText(/All questions answered/i)).toBeVisible({ timeout: 5000 })

    // Verify Generate button is visible and enabled
    const generateButton = page.getByRole('button', { name: /Generate Constitution/i })
    await expect(generateButton).toBeVisible()
    await expect(generateButton).toBeEnabled()

    // Verify state reflects completed questions
    const state = await getAppState(page)
    const workflow = state.projects[0]?.worktrees[0]?.tasks?.constitution_workflow
    expect(workflow).toBeDefined()
    expect(workflow.current_question).toBe(4)
    expect(workflow.status).toBe('collecting')

    // Note: We don't actually click Generate in CI because Claude CLI may not be available
    // and would cause the test to hang waiting for the process
    console.log('Generate Constitution button verified - skipping actual generation (Claude CLI required)')
  })

  test.skip('should create constitution.md file after generation', async ({ page }) => {
    // This requires Claude CLI to be installed
    // Skip in automated environments

    // TODO: Mock Claude CLI output for testing
  })
})
