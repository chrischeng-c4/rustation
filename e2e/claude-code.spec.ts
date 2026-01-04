import { test, expect } from './electron.fixture'
import path from 'path'
import { openProject } from './test-helpers'

test.describe('Claude Code Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Use the actual rustation project
    const testProjectPath = path.resolve(__dirname, '..')
    await openProject(page, testProjectPath)
    await page.waitForTimeout(1000)

    // Navigate to Chat tab
    const chatTab = page.getByRole('tab', { name: 'Chat' })
    await chatTab.click()
    await page.waitForTimeout(500)
  })

  // Helper to check if project is open
  async function hasProjectOpen(page: import('@playwright/test').Page): Promise<boolean> {
    const projectTabs = page.getByTestId('project-tabs')
    const isVisible = await projectTabs.isVisible({ timeout: 3000 }).catch(() => false)
    console.log(`[DEBUG] projectTabs isVisible: ${isVisible}`)
    return isVisible
  }

  // Helper to get chat input
  function getChatInput(page: import('@playwright/test').Page) {
    return page.getByPlaceholder(/Ask Claude about your project/i)
  }

  test('should display Chat page', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // Look for Chat header or input
    const chatInput = getChatInput(page)
    await expect(chatInput).toBeVisible({ timeout: 10000 })
  })

  test('chat input accepts text', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    const textarea = getChatInput(page)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Type a message
    await textarea.fill('Hello, this is a test message')

    // Verify the text was entered
    const value = await textarea.inputValue()
    expect(value).toContain('test message')
  })

  test('send button state changes based on input', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    const textarea = getChatInput(page)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Find the send button
    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()

    // Initially disabled (empty input)
    await expect(sendButton).toBeDisabled()

    // Type a message
    await textarea.fill('Test')

    // Now should be enabled
    await expect(sendButton).toBeEnabled()

    // Clear the input
    await textarea.fill('')

    // Should be disabled again
    await expect(sendButton).toBeDisabled()
  })

  // ============================================================================
  // Message Flow Tests - Verify actual message sending functionality
  // ============================================================================

  test('sending message adds user message to chat state', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    const textarea = getChatInput(page)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Type a unique test message
    const testMessage = `Test message ${Date.now()}`
    await textarea.fill(testMessage)

    // Click send button
    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Verify user message appears in state
    await page.waitForTimeout(1000)
    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      // active_project is not serialized, use projects[active_project_index]
      const activeProject = parsed.projects?.[parsed.active_project_index ?? 0]
      return activeProject?.worktrees?.[0]?.chat
    })

    // User message should be in state
    const hasUserMessage = chatState?.messages?.some(
      (m: { role: string; content: string }) => m.role === 'user' && m.content.includes('Test message')
    )
    expect(hasUserMessage).toBe(true)
  })

  test('sending message creates assistant placeholder and sets typing', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    const textarea = getChatInput(page)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    await textarea.fill('Hello Claude')

    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Check state immediately after sending
    await page.waitForTimeout(500)
    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      const activeProject = parsed.projects?.[parsed.active_project_index ?? 0]
      return activeProject?.worktrees?.[0]?.chat
    })

    // Should have both user and assistant messages (placeholder)
    const userMsg = chatState?.messages?.find((m: { role: string }) => m.role === 'user')
    const assistantMsg = chatState?.messages?.find((m: { role: string }) => m.role === 'assistant')

    expect(userMsg).toBeDefined()
    expect(assistantMsg).toBeDefined()
  })

  // ============================================================================
  // Error Handling and Timeout Tests
  // ============================================================================

  test('is_typing flag clears after timeout or error', async ({ page }) => {
    test.setTimeout(120000) // Extend timeout to 2 minutes

    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    const textarea = getChatInput(page)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Send a message
    await textarea.fill('Test timeout handling')
    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Wait briefly for message to be sent
    await page.waitForTimeout(2000)

    // Check initial state and debug logs
    let chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      const activeProject = parsed.projects?.[parsed.active_project_index ?? 0]
      return activeProject?.worktrees?.[0]?.chat
    })

    console.log('\n=== Initial State ===')
    console.log('is_typing:', chatState?.is_typing)
    console.log('error:', chatState?.error)

    expect(chatState?.is_typing).toBe(true)

    // Wait for timeout/completion (up to 45 seconds)
    // Backend timeout is 30s, adding more buffer for CI
    await page.waitForTimeout(45000)

    // Check is_typing is now false (should be cleared by timeout or completion)
    chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      const activeProject = parsed.projects?.[parsed.active_project_index ?? 0]
      return activeProject?.worktrees?.[0]?.chat
    })

    console.log('\n=== Final State ===')
    console.log('is_typing:', chatState?.is_typing)
    expect(chatState?.is_typing).toBe(false)
  })
})
