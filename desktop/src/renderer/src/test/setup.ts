import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock window.stateApi (state-first API)
const mockStateApi = {
  dispatch: vi.fn().mockResolvedValue(undefined),
  getState: vi.fn().mockResolvedValue('{}'),
  onStateUpdate: vi.fn().mockReturnValue(() => {}),
}

// Mock window.dialogApi
const mockDialogApi = {
  openFolder: vi.fn().mockResolvedValue(null),
}

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
  },
})

// Assign mocks to window
Object.defineProperty(window, 'stateApi', {
  value: mockStateApi,
  writable: true,
})

Object.defineProperty(window, 'dialogApi', {
  value: mockDialogApi,
  writable: true,
})

// Export mocks for test files to access
export { mockStateApi, mockDialogApi }

// Realistic ResizeObserver mock that simulates browser behavior
class ResizeObserverMock {
  private callback: ResizeObserverCallback
  private observedElements: Set<Element> = new Set()

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback
  }

  observe(element: Element) {
    this.observedElements.add(element)

    // Immediately trigger callback with realistic dimensions
    // This simulates what browser does after layout
    queueMicrotask(() => {
      this.callback(
        [{
          target: element,
          contentRect: {
            width: 800,
            height: 600,
            top: 0,
            left: 0,
            bottom: 600,
            right: 800,
            x: 0,
            y: 0,
          } as DOMRectReadOnly,
          borderBoxSize: [],
          contentBoxSize: [],
          devicePixelContentBoxSize: [],
        }] as ResizeObserverEntry[],
        this as ResizeObserver
      )
    })
  }

  unobserve(element: Element) {
    this.observedElements.delete(element)
  }

  disconnect() {
    this.observedElements.clear()
  }
}

global.ResizeObserver = ResizeObserverMock as any
