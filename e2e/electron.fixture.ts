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

    // Launch Electron app (headless mode - window won't show)
    const electronApp = await electron.launch({
      args: [
        path.join(appPath, 'out/main/index.js'),
        '--no-sandbox',
        '--enable-logging',
        '--v=1',
      ],
      cwd: appPath,
      env: {
        ...process.env,
        NODE_ENV: 'test',
        ELECTRON_ENABLE_LOGGING: '1',
        ELECTRON_ENABLE_STACK_DUMPING: '1',
        // Enable headless mode (skips mainWindow.show())
        HEADLESS: 'true',
        // Skip auto-open of recent project for clean test environment
        RSTN_TEST_MODE: '1',
      },
    })

    // Capture main process stdout/stderr for debugging
    electronApp.process().stdout?.on('data', (data: Buffer) => {
      console.log(`[MAIN stdout]`, data.toString().trim())
    })
    electronApp.process().stderr?.on('data', (data: Buffer) => {
      console.error(`[MAIN stderr]`, data.toString().trim())
    })

    // Use the app in tests
    await use(electronApp)

    // Cleanup
    await electronApp.close()
  },

  page: async ({ electronApp }, use) => {
    // Wait for the first window
    const page = await electronApp.firstWindow()

    // Capture console logs for debugging
    page.on('console', (msg) => {
      console.log(`[RENDERER ${msg.type()}]`, msg.text())
    })

    page.on('pageerror', (error) => {
      console.error('[RENDERER ERROR]', error.message)
    })

    // Wait for app to be ready
    await page.waitForLoadState('domcontentloaded')

    await use(page)
  },
})

export { expect } from '@playwright/test'
