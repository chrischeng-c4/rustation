import { test as base, _electron as electron, ElectronApplication, Page } from '@playwright/test'
import path from 'path'

// Extend base test with Electron fixtures
export const test = base.extend<{
  electronApp: ElectronApplication
  page: Page
}>({
  electronApp: async ({}, use) => {
    // Path to the built Electron app
    const appPath = path.join(__dirname, '../apps/desktop')

    // Launch Electron app
    const electronApp = await electron.launch({
      args: [path.join(appPath, 'out/main/index.js')],
      cwd: appPath,
      env: {
        ...process.env,
        NODE_ENV: 'test',
      },
    })

    // Use the app in tests
    await use(electronApp)

    // Cleanup
    await electronApp.close()
  },

  page: async ({ electronApp }, use) => {
    // Wait for the first window
    const page = await electronApp.firstWindow()

    // Wait for app to be ready
    await page.waitForLoadState('domcontentloaded')

    await use(page)
  },
})

export { expect } from '@playwright/test'
