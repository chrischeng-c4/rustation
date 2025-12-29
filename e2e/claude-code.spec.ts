import { test, expect } from './electron.fixture'

test.describe('Claude Code Integration', () => {
  // Helper to check if project is open
  async function hasProjectOpen(page: import('@playwright/test').Page): Promise<boolean> {
    const tasksHeading = page.locator('h2', { hasText: 'Tasks' })
    return await tasksHeading.isVisible({ timeout: 3000 }).catch(() => false)
  }

  // Helper to click the Claude Code button (uses unique violet border class)
  async function clickClaudeCode(page: import('@playwright/test').Page): Promise<void> {
    const claudeCodeButton = page.locator('button.border-violet-300')
    await claudeCodeButton.click()
    await page.waitForTimeout(500)
  }

  test('should display Claude Code in command list', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // Look for Claude Code label in the command list
    const claudeCodeLabel = page.getByText('Claude Code', { exact: true }).first()
    await expect(claudeCodeLabel).toBeVisible({ timeout: 5000 })

    // Also verify the description
    const description = page.getByText('Chat with Claude Code AI assistant')
    await expect(description).toBeVisible()
  })

  test('Claude Code appears first in command list with Bot icon', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // The Claude Code card should be in the Commands section
    const commandsSection = page.locator('text=Commands').first()
    await expect(commandsSection).toBeVisible()

    // Claude Code button should have violet styling (unique identifier)
    const claudeCodeButton = page.locator('button.border-violet-300')
    await expect(claudeCodeButton).toBeVisible()
  })

  test('clicking Claude Code shows chat panel', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // Wait for state to fully load
    await page.waitForTimeout(1000)

    // Click Claude Code button
    await clickClaudeCode(page)

    // The chat placeholder should be visible
    const chatInput = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(chatInput).toBeVisible({ timeout: 5000 })
  })

  test('chat input accepts text', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // Click Claude Code to show chat panel
    await clickClaudeCode(page)

    // Find the textarea
    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
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

    // Click Claude Code
    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Find the send button - it's inside the chat panel's input area
    // The ChatPanel has a Send button that's the last button in the input area
    const chatPanelButtons = page.locator('button').filter({ has: page.locator('svg.lucide-send') })
    const sendButton = chatPanelButtons.first()

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

  test('clicking Claude Code multiple times keeps chat panel visible', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    // Click Claude Code
    await clickClaudeCode(page)

    // Verify chat panel is shown
    const chatInput = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(chatInput).toBeVisible({ timeout: 3000 })

    // Click Claude Code again
    await clickClaudeCode(page)

    // Chat panel should still be visible (idempotent)
    await expect(chatInput).toBeVisible({ timeout: 3000 })

    // Verify the active command is still claude-code
    const state = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.tasks?.active_command
    })
    expect(state).toBe('claude-code')
  })

  // ============================================================================
  // Message Flow Tests - Verify actual message sending functionality
  // ============================================================================

  test('sending message adds user message to chat state', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    // Type a unique test message
    const testMessage = `Test message ${Date.now()}`
    await textarea.fill(testMessage)

    // Click send button
    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Verify user message appears in state
    await page.waitForTimeout(500)
    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
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

    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    await textarea.fill('Hello Claude')

    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Check state immediately after sending
    await page.waitForTimeout(200)
    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
    })

    // Should have both user and assistant messages (placeholder)
    const userMsg = chatState?.messages?.find((m: { role: string }) => m.role === 'user')
    const assistantMsg = chatState?.messages?.find((m: { role: string }) => m.role === 'assistant')

    expect(userMsg).toBeDefined()
    expect(assistantMsg).toBeDefined()
  })

  test('chat state contains messages after send', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
    await expect(textarea).toBeVisible({ timeout: 3000 })

    await textarea.fill('Test message')

    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Wait briefly for state updates
    await page.waitForTimeout(1000)

    // Verify chat state has messages
    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
    })

    // Should have at least 2 messages (user + assistant placeholder)
    expect(chatState?.messages?.length).toBeGreaterThanOrEqual(2)

    // Verify user message exists
    const userMessage = chatState?.messages?.find(
      (m: { role: string }) => m.role === 'user'
    )
    expect(userMessage).toBeDefined()

    // Verify assistant message exists (placeholder or with content)
    const assistantMessage = chatState?.messages?.find(
      (m: { role: string }) => m.role === 'assistant'
    )
    expect(assistantMessage).toBeDefined()
  })

  // ============================================================================
  // Error Handling and Timeout Tests
  // ============================================================================

  test('is_typing flag clears after timeout or error', async ({ page }) => {
    test.setTimeout(120000) // Extend timeout to 2 minutes

    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
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
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
    })

    console.log('\n=== Initial State ===')
    console.log('is_typing:', chatState?.is_typing)
    console.log('error:', chatState?.error)

    expect(chatState?.is_typing).toBe(true)

    // Wait for timeout/completion (up to 35 seconds)
    await page.waitForTimeout(35000)

    // Check is_typing is now false (should be cleared by timeout or completion)
    chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
    })

    console.log('\n=== Final State ===')
    console.log('is_typing:', chatState?.is_typing)
    console.log('error:', chatState?.error)
    console.log('messages:', chatState?.messages?.length || 0)

    // CRITICAL: is_typing MUST be false after timeout
    expect(chatState?.is_typing).toBe(false)
  })

  test('error message appears when Claude CLI not available', async ({ page }) => {
    const hasProject = await hasProjectOpen(page)
    test.skip(!hasProject, 'No project available for testing')

    await clickClaudeCode(page)

    const textarea = page.getByPlaceholder(/Ask Claude Code/i)
    await textarea.fill('Test error handling')

    const sendButton = page.locator('button').filter({ has: page.locator('svg.lucide-send') }).first()
    await sendButton.click()

    // Wait for error to appear (if Claude CLI not available)
    await page.waitForTimeout(3000)

    const chatState = await page.evaluate(async () => {
      const json = await (window as any).stateApi.getState()
      const parsed = JSON.parse(json)
      return parsed.projects?.[0]?.worktrees?.[0]?.chat
    })

    // If there's an error, verify it's actionable (contains helpful information)
    if (chatState?.error) {
      const error = chatState.error.toLowerCase()
      // Error should mention the problem
      const isActionable = error.includes('claude') || error.includes('not found') || error.includes('timeout')
      expect(isActionable).toBe(true)
    }

    // Regardless of error, is_typing should be false
    expect(chatState?.is_typing).toBe(false)
  })
})
