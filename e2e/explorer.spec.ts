import { test, expect } from './electron.fixture'

test.describe('File Explorer', () => {
  // Navigate to Explorer view
  async function navigateToExplorer(page: import('@playwright/test').Page): Promise<void> {
    const explorerButton = page.getByRole('button', { name: 'Explorer' })
    await explorerButton.click()
    await page.waitForTimeout(500)
  }

  test('should display the Explorer page', async ({ page }) => {
    await navigateToExplorer(page)

    // Check for Explorer heading
    const heading = page.getByRole('heading', { name: 'File Explorer' })
    await expect(heading).toBeVisible()
  })

  test('should auto-load files on tab entry', async ({ page }) => {
    await navigateToExplorer(page)

    // Wait for file list to load
    await page.waitForTimeout(1000)

    // Should show item count in header (indicates files loaded)
    const itemCount = page.getByText(/\d+ items/i).first()
    await expect(itemCount).toBeVisible()

    // Files should be loaded without clicking any button
    const fileRows = page.getByText(/\.(json|toml|md|rs|tsx?)/i).first()
    await expect(fileRows).toBeVisible()
  })

  test('should show home button in tree header', async ({ page }) => {
    await navigateToExplorer(page)

    // Check for Home icon button in tree view header
    const homeButton = page.locator('[data-testid="HomeIcon"]').first()
    await expect(homeButton).toBeVisible()
  })

  test.describe('Tree View', () => {
    test('should show project name in tree header', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(500)

      // The tree header should show the project name (last segment of path)
      // For rustation project, it should show "rustation"
      const projectName = page.getByText('rustation').first()
      await expect(projectName).toBeVisible()
    })

    test('should show item count in tree header', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Check for item count
      const itemCount = page.getByText(/\d+ items/i).first()
      await expect(itemCount).toBeVisible()
    })

    test('should show folders with expand arrows', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Look for common folders with expand arrows
      const folderNames = ['src', 'packages', 'desktop', 'docs', 'e2e', 'openspec']

      let folderFound = false
      for (const folderName of folderNames) {
        const folder = page.getByText(folderName, { exact: true }).first()
        const isVisible = await folder.isVisible().catch(() => false)

        if (isVisible) {
          // Folder should have an expand arrow (ChevronRight icon)
          const chevronIcon = page.locator('[data-testid="ChevronRightIcon"]').first()
          const iconVisible = await chevronIcon.isVisible().catch(() => false)

          if (iconVisible) {
            folderFound = true
            break
          }
        }
      }

      expect(folderFound).toBe(true)
    })

    test('should expand folder when clicking arrow', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Find the first expand arrow and click it
      const expandArrow = page.locator('[data-testid="ChevronRightIcon"]').first()
      const isVisible = await expandArrow.isVisible().catch(() => false)

      test.skip(!isVisible, 'No expandable folder found')

      // Get the parent button and click
      const expandButton = expandArrow.locator('xpath=ancestor::button[1]')
      await expandButton.click()
      await page.waitForTimeout(500)

      // After expansion, should see ExpandMore icon instead of ChevronRight
      const expandMoreIcon = page.locator('[data-testid="ExpandMoreIcon"]').first()
      await expect(expandMoreIcon).toBeVisible()
    })

    test('should show nested files after expanding folder', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Find a known folder like 'packages' and expand it
      const packagesFolder = page.getByText('packages', { exact: true }).first()
      const isVisible = await packagesFolder.isVisible().catch(() => false)

      test.skip(!isVisible, 'packages folder not found')

      // Find the expand button near the packages folder
      const folderRow = packagesFolder.locator('xpath=ancestor::div[contains(@class, "MuiBox-root")][1]')
      const expandButton = folderRow.locator('button').first()
      await expandButton.click()
      await page.waitForTimeout(500)

      // Should show 'core' subfolder after expansion
      const coreFolder = page.getByText('core', { exact: true }).first()
      await expect(coreFolder).toBeVisible()
    })

    test('should collapse folder when clicking expanded arrow', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Find and expand a folder first
      const expandArrow = page.locator('[data-testid="ChevronRightIcon"]').first()
      const isVisible = await expandArrow.isVisible().catch(() => false)

      test.skip(!isVisible, 'No expandable folder found')

      // Expand
      const expandButton = expandArrow.locator('xpath=ancestor::button[1]')
      await expandButton.click()
      await page.waitForTimeout(500)

      // Now collapse
      const collapseArrow = page.locator('[data-testid="ExpandMoreIcon"]').first()
      const collapseButton = collapseArrow.locator('xpath=ancestor::button[1]')
      await collapseButton.click()
      await page.waitForTimeout(300)

      // Should see ChevronRight again
      const chevronIcon = page.locator('[data-testid="ChevronRightIcon"]').first()
      await expect(chevronIcon).toBeVisible()
    })
  })

  test.describe('File Selection', () => {
    test('should select file when clicking on it', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Click on a file
      const fileNames = ['package.json', 'Cargo.toml', 'README.md', 'tsconfig.json']
      let fileSelected = false

      for (const fileName of fileNames) {
        const file = page.getByText(fileName, { exact: true }).first()
        const isVisible = await file.isVisible().catch(() => false)

        if (isVisible) {
          await file.click()
          await page.waitForTimeout(500)
          fileSelected = true
          break
        }
      }

      test.skip(!fileSelected, 'No suitable file found for selection test')

      // File should be highlighted (selected styling applied)
      // The selected file should have bold font weight
      const selectedFile = page.locator('[style*="font-weight: 600"]').first()
      const hasSelection = await selectedFile.isVisible().catch(() => false)
      expect(hasSelection).toBe(true)
    })

    test('should show file preview when selecting a file', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Select a file
      const file = page.getByText('package.json', { exact: true }).first()
      const isVisible = await file.isVisible().catch(() => false)

      test.skip(!isVisible, 'package.json not found')

      await file.click()
      await page.waitForTimeout(1000)

      // Preview panel should show file content
      // package.json should have "name" and "version" fields
      const nameField = page.getByText('"name"').first()
      await expect(nameField).toBeVisible()
    })
  })

  test.describe('Home Button Navigation', () => {
    test('should navigate to root when clicking home button', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // First, expand a folder to change state
      const expandArrow = page.locator('[data-testid="ChevronRightIcon"]').first()
      const isVisible = await expandArrow.isVisible().catch(() => false)

      if (isVisible) {
        const expandButton = expandArrow.locator('xpath=ancestor::button[1]')
        await expandButton.click()
        await page.waitForTimeout(500)
      }

      // Click home button
      const homeButton = page.locator('[data-testid="HomeIcon"]').first()
      await homeButton.click()
      await page.waitForTimeout(500)

      // Should still show root files
      const itemCount = page.getByText(/\d+ items/i).first()
      await expect(itemCount).toBeVisible()
    })
  })

  test.describe('Comment Functionality', () => {
    test('should add inline comments via line number click', async ({ page }) => {
      await navigateToExplorer(page)
      await page.waitForTimeout(1000)

      // Select a file (try to find a common file)
      const fileNames = ['package.json', 'Cargo.toml', 'README.md', 'tsconfig.json']
      let fileSelected = false

      for (const fileName of fileNames) {
        const file = page.getByText(fileName, { exact: true }).first()
        const isVisible = await file.isVisible().catch(() => false)

        if (isVisible) {
          await file.click()
          await page.waitForTimeout(500)
          fileSelected = true
          break
        }
      }

      test.skip(!fileSelected, 'No suitable file found for comment test')

      // Wait for file preview to load
      await page.waitForTimeout(500)

      // Look for line numbers in the preview (new inline comment system)
      const lineNumber = page.locator('span').filter({ hasText: /^1$/ }).first()
      const lineVisible = await lineNumber.isVisible().catch(() => false)

      if (lineVisible) {
        // Hover over line to show add comment button
        await lineNumber.hover()
        await page.waitForTimeout(200)

        // Click add comment button if visible
        const addCommentBtn = page.getByRole('button', { name: /Add comment/i }).first()
        const btnVisible = await addCommentBtn.isVisible().catch(() => false)

        if (btnVisible) {
          await addCommentBtn.click()
          await page.waitForTimeout(200)

          // Type comment
          const testComment = `E2E Test Comment - ${Date.now()}`
          const commentInput = page.getByPlaceholder(/Add comment on line/i)

          if (await commentInput.isVisible().catch(() => false)) {
            await commentInput.fill(testComment)

            // Submit with send button
            const sendButton = page.getByRole('button').filter({ has: page.locator('svg') }).first()
            await sendButton.click()
            await page.waitForTimeout(500)

            // Verify comment appears
            const commentText = page.getByText(testComment)
            const commentVisible = await commentText.isVisible().catch(() => false)
            expect(commentVisible).toBe(true)
          }
        }
      }
    })
  })
})
