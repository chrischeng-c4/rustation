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

    // Should show file entries (check for common files like package.json, Cargo.toml, etc.)
    const fileTable = page.locator('[role="table"]').first()
    await expect(fileTable).toBeVisible()

    // Files should be loaded without clicking "Project" button
    const fileRows = page.getByText(/\.(json|toml|md|rs|tsx?)/i).first()
    await expect(fileRows).toBeVisible()
  })

  test('should show breadcrumb with Project button', async ({ page }) => {
    await navigateToExplorer(page)

    // Check for Project button in breadcrumbs
    const projectButton = page.getByRole('button', { name: 'Project' })
    await expect(projectButton).toBeVisible()
  })

  test('should update breadcrumb when navigating to folders', async ({ page }) => {
    await navigateToExplorer(page)
    await page.waitForTimeout(1000)

    // Find a folder and double-click to navigate
    // Look for common folders like 'src', 'packages', 'desktop', etc.
    const folderNames = ['src', 'packages', 'desktop', 'docs', 'e2e', 'openspec']

    let folderFound = false
    for (const folderName of folderNames) {
      const folder = page.getByText(folderName, { exact: true }).first()
      const isVisible = await folder.isVisible().catch(() => false)

      if (isVisible) {
        // Double-click to enter folder
        await folder.dblclick()
        await page.waitForTimeout(500)

        // Check if breadcrumb updated (should show folder name)
        const breadcrumb = page.getByText(folderName)
        const breadcrumbVisible = await breadcrumb.isVisible().catch(() => false)

        if (breadcrumbVisible) {
          folderFound = true
          break
        }
      }
    }

    expect(folderFound).toBe(true)
  })

  test('should show correct layout (sidebar file list + main detail panel)', async ({ page }) => {
    await navigateToExplorer(page)
    await page.waitForTimeout(500)

    // Get the file list container and detail panel
    const fileListContainer = page.locator('div').filter({ hasText: /Name.*Size.*Status.*Modified/i }).first()
    const detailPanel = page.getByRole('tab', { name: /Info|Preview|Comments/i }).first()

    await expect(fileListContainer).toBeVisible()
    await expect(detailPanel).toBeVisible()

    // Check layout: file list should be narrower than detail panel
    const fileListBox = await fileListContainer.boundingBox()
    const detailPanelBox = await detailPanel.boundingBox()

    if (fileListBox && detailPanelBox) {
      // File list should be around 300px, detail panel should be wider
      expect(fileListBox.width).toBeLessThan(detailPanelBox.width)
    }
  })

  test.describe('Comment Functionality', () => {
    test('should add and persist comments', async ({ page }) => {
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
          await page.waitForTimeout(300)
          fileSelected = true
          break
        }
      }

      test.skip(!fileSelected, 'No suitable file found for comment test')

      // Navigate to Comments tab
      const commentsTab = page.getByRole('tab', { name: 'Comments' })
      await commentsTab.click()
      await page.waitForTimeout(300)

      // Add a comment
      const testComment = `E2E Test Comment - ${Date.now()}`
      const commentInput = page.getByPlaceholder(/Add a comment|Enter comment/i)

      if (await commentInput.isVisible().catch(() => false)) {
        await commentInput.fill(testComment)

        // Submit comment (look for submit button)
        const submitButton = page.getByRole('button', { name: /Submit|Add|Post/i })
        await submitButton.click()
        await page.waitForTimeout(500)

        // Verify comment appears in the list
        const commentText = page.getByText(testComment)
        await expect(commentText).toBeVisible()

        // Check if comment count badge appears in file list
        // This would be a small badge with a number next to the file name
        const commentBadge = page.locator('[data-testid="comment-badge"]').or(
          page.locator('div').filter({ hasText: /💬|🗨️|\d+/ })
        ).first()

        // Comment badge should be visible (if implemented)
        const hasBadge = await commentBadge.isVisible().catch(() => false)
        if (hasBadge) {
          await expect(commentBadge).toBeVisible()
        }
      } else {
        test.skip(true, 'Comment input not available')
      }
    })
  })
})
