import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

// Dialog API for native dialogs
const dialogApi = {
  /**
   * Open a native folder selection dialog.
   * @returns The selected folder path, or null if canceled
   */
  openFolder: (): Promise<string | null> => {
    return ipcRenderer.invoke('dialog:openFolder')
  },
}

// Screenshot API (dev mode)
const screenshotApi = {
  /**
   * Capture a screenshot of the entire window and save to Downloads folder.
   * @returns Result object with success flag and file path or error message
   */
  capture: (): Promise<{ success: boolean; filePath?: string; error?: string }> => {
    return ipcRenderer.invoke('screenshot:capture')
  },
}

// State-first API
// This is the new architecture where Rust owns all state
const stateApi = {
  /**
   * Dispatch an action to update state.
   * @param action - Action object (will be JSON serialized)
   */
  dispatch: (action: unknown): Promise<void> => {
    return ipcRenderer.invoke('state:dispatch', JSON.stringify(action))
  },

  /**
   * Get the current state.
   * @returns JSON string of the current state
   */
  getState: (): Promise<string> => {
    return ipcRenderer.invoke('state:get')
  },

  /**
   * Subscribe to state updates.
   * @param callback - Called with JSON string whenever state changes
   * @returns Unsubscribe function
   */
  onStateUpdate: (callback: (stateJson: string) => void): (() => void) => {
    const handler = (_event: Electron.IpcRendererEvent, stateJson: string): void => {
      callback(stateJson)
    }
    ipcRenderer.on('state:update', handler)
    return () => {
      ipcRenderer.removeListener('state:update', handler)
    }
  },
}

// Expose electron APIs to renderer
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    contextBridge.exposeInMainWorld('stateApi', stateApi)
    contextBridge.exposeInMainWorld('dialogApi', dialogApi)
    contextBridge.exposeInMainWorld('screenshotApi', screenshotApi)
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI
  // @ts-ignore (define in dts)
  window.stateApi = stateApi
  // @ts-ignore (define in dts)
  window.dialogApi = dialogApi
  // @ts-ignore (define in dts)
  window.screenshotApi = screenshotApi
}
