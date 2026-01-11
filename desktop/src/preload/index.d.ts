import { ElectronAPI } from '@electron-toolkit/preload'

// Docker service types (matching Rust DockerService struct)
interface DockerService {
  id: string
  name: string
  image: string
  status: 'running' | 'stopped' | 'starting'
  port: number
  service_type: 'Database' | 'Cache' | 'MessageBroker' | 'Other'
}

// Justfile command types (matching Rust JustCommand struct)
interface JustCommand {
  name: string
  description: string | null
  recipe: string
}

// Branch info types (matching Rust BranchInfo struct)
interface BranchInfo {
  name: string
  hasWorktree: boolean
  isCurrent: boolean
}

// Dialog API for native dialogs
interface DialogApi {
  /**
   * Open a native folder selection dialog.
   * @returns The selected folder path, or null if canceled
   */
  openFolder(): Promise<string | null>
}

// Screenshot API (dev mode)
interface ScreenshotApi {
  /**
   * Capture a screenshot of the entire window and save to Downloads folder.
   * @returns Result object with success flag and file path or error message
   */
  capture(): Promise<{ success: boolean; filePath?: string; error?: string }>
}

// State-first API
// This is the new architecture where Rust owns all state
interface StateApi {
  /**
   * Dispatch an action to update state.
   * @param action - Action object (will be JSON serialized)
   */
  dispatch(action: unknown): Promise<void>

  /**
   * Get the current state.
   * @returns JSON string of the current state
   */
  getState(): Promise<string>

  /**
   * Subscribe to state updates.
   * @param callback - Called with JSON string whenever state changes
   * @returns Unsubscribe function
   */
  onStateUpdate(callback: (stateJson: string) => void): () => void
}

declare global {
  interface Window {
    electron: ElectronAPI
    stateApi: StateApi
    dialogApi: DialogApi
    screenshotApi: ScreenshotApi
  }
}
