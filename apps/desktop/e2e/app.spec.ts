import { test, expect, _electron as electron } from '@playwright/test'
import path from 'path'
import os from 'os'
import fs from 'fs'

let electronApp: Awaited<ReturnType<typeof electron.launch>>
let tempDir: string

test.beforeAll(async () => {
  // Create a temp directory to avoid polluting real state
  tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rstn-e2e-app-'))

  // Build the app first
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
    env: {
      ...process.env,
      RSTN_DATA_DIR: tempDir,
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

test('app launches and shows no project view', async () => {
  const window = await electronApp.firstWindow()

  // Wait for the app to load
  await window.waitForLoadState('domcontentloaded')
  await window.waitForTimeout(2000)

  // Should show "No Project Open" when no projects are open
  const heading = window.locator('text=No Project Open')
  await expect(heading).toBeVisible({ timeout: 10000 })
})

test('has open project button', async () => {
  const window = await electronApp.firstWindow()
  await window.waitForTimeout(1000)

  // Should have at least one Open Project button
  const openButtons = window.getByRole('button', { name: 'Open Project' })
  await expect(openButtons.first()).toBeVisible({ timeout: 5000 })
})
