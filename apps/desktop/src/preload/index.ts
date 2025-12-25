import { contextBridge } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

// Expose electron APIs to renderer
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    // We'll expose the Rust core API here once napi-rs is set up
    contextBridge.exposeInMainWorld('api', {
      // Placeholder - will be replaced with napi-rs bindings
      ping: () => 'pong'
    })
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI
  // @ts-ignore (define in dts)
  window.api = { ping: () => 'pong' }
}
