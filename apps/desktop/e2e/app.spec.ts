import { test, expect, _electron as electron } from '@playwright/test'
import path from 'path'

let electronApp: Awaited<ReturnType<typeof electron.launch>>

test.beforeAll(async () => {
  // Build the app first
  electronApp = await electron.launch({
    args: [path.join(__dirname, '../out/main/index.js')],
  })
})

test.afterAll(async () => {
  await electronApp.close()
})

test('app launches and shows no project view', async () => {
  const window = await electronApp.firstWindow()

  // Wait for the app to load
  await window.waitForLoadState('domcontentloaded')

  // Should show "No Project Open" when no projects are open
  const heading = window.locator('text=No Project Open')
  await expect(heading).toBeVisible({ timeout: 10000 })
})

test('has open project button', async () => {
  const window = await electronApp.firstWindow()

  // Should have at least one Open Project button
  const openButtons = window.getByRole('button', { name: 'Open Project' })
  await expect(openButtons.first()).toBeVisible()
})
