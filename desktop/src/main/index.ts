import { app, shell, BrowserWindow, ipcMain, dialog, clipboard } from 'electron'
import { join, resolve } from 'path'
import { existsSync } from 'fs'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'
import * as core from '@rstn/core'

// Track the main window for state updates
let mainWindow: BrowserWindow | null = null

// Project path passed via CLI (e.g., `rstn /path/to/project`)
let cliProjectPath: string | null = null

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    show: false,
    autoHideMenuBar: true,
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false
    }
  })

  mainWindow.on('ready-to-show', () => {
    // Skip showing window in headless test mode
    if (process.env.NODE_ENV === 'test' && process.env.HEADLESS === 'true') {
      return
    }
    mainWindow?.show()
  })

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  // Load the app
  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }
}

// ============================================================================
// State Management (State-first architecture)
// ============================================================================

function initializeState(): void {
  // Initialize state with a callback that forwards updates to renderer
  core.stateInit((err: Error | null, stateJson: string) => {
    if (err) {
      console.error('State update error:', err)
      return
    }
    // Forward state updates to all windows
    BrowserWindow.getAllWindows().forEach((win) => {
      win.webContents.send('state:update', stateJson)
    })
  })
}

// IPC Handlers for state management
function setupStateIPC(): void {
  // Handle state dispatch from renderer
  ipcMain.handle('state:dispatch', async (_event, actionJson: string) => {
    try {
      await core.stateDispatch(actionJson)
    } catch (error) {
      console.error('Dispatch error:', error)
      throw error
    }
  })

  // Handle state get request from renderer
  ipcMain.handle('state:get', async () => {
    try {
      return await core.stateGet()
    } catch (error) {
      console.error('State get error:', error)
      throw error
    }
  })
}

// ============================================================================
// Explorer Handlers
// ============================================================================

function setupExplorerIPC(): void {
  // List directory entries without mutating state (for tree view expansion)
  ipcMain.handle('explorer:listDirectory', async (_event, path: string, projectRoot: string) => {
    try {
      return core.explorerListDirectory(path, projectRoot)
    } catch (error) {
      console.error('Explorer list directory error:', error)
      throw error
    }
  })
}

// ============================================================================
// Dialog Handlers
// ============================================================================

function setupDialogIPC(): void {
  // Open folder dialog
  ipcMain.handle('dialog:openFolder', async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory'],
      title: 'Open Project Folder',
    })
    if (result.canceled || result.filePaths.length === 0) {
      return null
    }
    return result.filePaths[0]
  })
}

// ============================================================================
// Screenshot Handler (Dev Mode)
// ============================================================================

function setupScreenshotIPC(): void {
  ipcMain.handle('screenshot:capture', async () => {
    if (!mainWindow) {
      return { success: false, error: 'No main window' }
    }

    try {
      // Capture the entire window as PNG
      const image = await mainWindow.webContents.capturePage()

      // Generate filename with timestamp
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
      const filename = `rstn-screenshot-${timestamp}.png`

      // Get downloads folder path
      const downloadsPath = app.getPath('downloads')
      const filePath = join(downloadsPath, filename)

      // Write image to file
      const { promises: fs } = await import('fs')
      await fs.writeFile(filePath, image.toPNG())

      // Copy image to clipboard
      try {
        clipboard.writeImage(image)
        console.log('Screenshot saved to:', filePath, '(and copied to clipboard)')
      } catch (clipboardError) {
        console.warn('Clipboard copy failed:', clipboardError)
      }

      return { success: true, filePath }
    } catch (error) {
      console.error('Screenshot failed:', error)
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      }
    }
  })
}

// ============================================================================
// CLI Argument Parsing
// ============================================================================

function parseCliArguments(): void {
  // Get arguments after the electron executable
  // In dev: electron . [path]
  // In production: rstn-desktop [path]
  const args = process.argv.slice(is.dev ? 2 : 1)

  // Find the first argument that looks like a path (not a flag)
  const pathArg = args.find((arg) => !arg.startsWith('-') && !arg.startsWith('--'))

  if (pathArg) {
    const resolvedPath = resolve(pathArg)
    if (existsSync(resolvedPath)) {
      cliProjectPath = resolvedPath
      console.log(`CLI: Opening project at ${cliProjectPath}`)
    } else {
      console.warn(`CLI: Path does not exist: ${pathArg}`)
    }
  }
}

// Open project from CLI after state is initialized
async function openCliProject(): Promise<void> {
  if (cliProjectPath) {
    try {
      await core.stateDispatch(JSON.stringify({
        type: 'OpenProject',
        payload: { path: cliProjectPath }
      }))
    } catch (error) {
      console.error('Failed to open CLI project:', error)
    }
  }
}

// Parse CLI arguments early
parseCliArguments()

app.whenReady().then(async () => {
  electronApp.setAppUserModelId('com.rstn.desktop')

  // Initialize state management (State-first architecture)
  initializeState()
  setupStateIPC()
  setupExplorerIPC()
  setupDialogIPC()
  setupScreenshotIPC()

  // Open project from CLI if provided
  await openCliProject()

  // Watch for shortcuts in development
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window)
  })

  createWindow()

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
