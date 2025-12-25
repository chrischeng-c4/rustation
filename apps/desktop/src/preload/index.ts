import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'
import * as core from '@rstn/core'

// Build the API object with all napi-rs functions
// NOTE: This is the legacy API (React-first). Use stateApi for new code.
const api = {
  docker: {
    isAvailable: () => core.dockerIsAvailable(),
    listServices: () => core.dockerListServices(),
    startService: (id: string) => core.dockerStartService(id),
    stopService: (id: string) => core.dockerStopService(id),
    restartService: (id: string) => core.dockerRestartService(id),
    getLogs: (id: string, tail?: number) => core.dockerGetLogs(id, tail),
    removeService: (id: string) => core.dockerRemoveService(id),
    createDatabase: (id: string, dbName: string) => core.dockerCreateDatabase(id, dbName),
    createVhost: (id: string, vhostName: string) => core.dockerCreateVhost(id, vhostName),
  },
  justfile: {
    parse: (path: string) => core.justfileParse(path),
    run: (command: string, cwd: string) => core.justfileRun(command, cwd),
  },
}

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
    contextBridge.exposeInMainWorld('api', api)
    contextBridge.exposeInMainWorld('stateApi', stateApi)
    contextBridge.exposeInMainWorld('dialogApi', dialogApi)
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI
  // @ts-ignore (define in dts)
  window.api = api
  // @ts-ignore (define in dts)
  window.stateApi = stateApi
  // @ts-ignore (define in dts)
  window.dialogApi = dialogApi
}
