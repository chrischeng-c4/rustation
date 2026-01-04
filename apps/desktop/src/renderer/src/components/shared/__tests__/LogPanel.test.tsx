import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { LogPanel } from '../LogPanel'

describe('LogPanel', () => {
  it('renders with default title', () => {
    render(<LogPanel logs={[]} />)
    expect(screen.getByText('Output')).toBeInTheDocument()
  })

  it('renders with custom title', () => {
    render(<LogPanel title="Custom Title" logs={[]} />)
    expect(screen.getByText('Custom Title')).toBeInTheDocument()
  })

  it('shows empty message when no logs', () => {
    render(<LogPanel logs={[]} emptyMessage="Nothing here" />)
    expect(screen.getByText('Nothing here')).toBeInTheDocument()
  })

  it('shows default empty message', () => {
    render(<LogPanel logs={[]} />)
    expect(screen.getByText('No output')).toBeInTheDocument()
  })

  it('renders logs', () => {
    const logs = ['line 1', 'line 2', 'line 3']
    render(<LogPanel logs={logs} />)
    // Logs are joined with newlines and rendered in a pre tag
    const preElement = screen.getByText(/line 1/)
    expect(preElement).toBeInTheDocument()
    expect(preElement.textContent).toContain('line 2')
    expect(preElement.textContent).toContain('line 3')
  })

  it('shows copy button when logs exist and showCopy is true', () => {
    render(<LogPanel logs={['test']} showCopy={true} />)
    // Copy button should be visible (lucide-react Copy icon)
    const buttons = screen.getAllByRole('button')
    expect(buttons.length).toBeGreaterThan(0)
  })

  it('hides copy button when showCopy is false', () => {
    render(<LogPanel logs={['test']} showCopy={false} />)
    // Should not have copy button
    const buttons = screen.queryAllByRole('button')
    expect(buttons.length).toBe(0)
  })

  it('hides copy button when no logs', () => {
    render(<LogPanel logs={[]} showCopy={true} />)
    const buttons = screen.queryAllByRole('button')
    expect(buttons.length).toBe(0)
  })

  it('shows refresh button when onRefresh is provided', () => {
    const onRefresh = vi.fn()
    render(<LogPanel logs={[]} onRefresh={onRefresh} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons.length).toBe(1)
  })

  it('calls onRefresh when refresh button clicked', () => {
    const onRefresh = vi.fn()
    render(<LogPanel logs={[]} onRefresh={onRefresh} />)
    const refreshButton = screen.getByRole('button')
    fireEvent.click(refreshButton)
    expect(onRefresh).toHaveBeenCalledTimes(1)
  })

  it('disables refresh button when isRefreshing', () => {
    const onRefresh = vi.fn()
    render(<LogPanel logs={[]} onRefresh={onRefresh} isRefreshing={true} />)
    const refreshButton = screen.getByRole('button')
    expect(refreshButton).toBeDisabled()
  })

  it('copies logs to clipboard when copy button clicked', async () => {
    const logs = ['line 1', 'line 2']
    render(<LogPanel logs={logs} showCopy={true} />)

    const buttons = screen.getAllByRole('button')
    const copyButton = buttons[0]
    fireEvent.click(copyButton)

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('line 1\nline 2')
    })
  })
})
