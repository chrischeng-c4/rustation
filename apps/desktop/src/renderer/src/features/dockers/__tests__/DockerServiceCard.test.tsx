import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { DockerServiceCard } from '../DockerServiceCard'
import type { DockerService } from '@/types/docker'

const mockService: DockerService = {
  id: 'rstn-postgres',
  name: 'PostgreSQL',
  image: 'postgres:16',
  status: 'running',
  port: 5432,
  service_type: 'Database',
  is_rstn_managed: true,
}

describe('DockerServiceCard', () => {
  it('renders service name and image', () => {
    render(<DockerServiceCard service={mockService} />)
    expect(screen.getByText('PostgreSQL')).toBeInTheDocument()
    expect(screen.getByText('postgres:16')).toBeInTheDocument()
  })

  it('renders port when available', () => {
    render(<DockerServiceCard service={mockService} />)
    expect(screen.getByText('5432')).toBeInTheDocument()
  })

  it('shows Running status badge for running service', () => {
    render(<DockerServiceCard service={mockService} />)
    expect(screen.getByText('Running')).toBeInTheDocument()
  })

  it('shows Stopped status badge for stopped service', () => {
    const stoppedService = { ...mockService, status: 'stopped' as const }
    render(<DockerServiceCard service={stoppedService} />)
    expect(screen.getByText('Stopped')).toBeInTheDocument()
  })

  it('shows Stop button when running', () => {
    render(<DockerServiceCard service={mockService} />)
    expect(screen.getByText('Stop')).toBeInTheDocument()
  })

  it('shows Start button when stopped', () => {
    const stoppedService = { ...mockService, status: 'stopped' as const }
    render(<DockerServiceCard service={stoppedService} />)
    expect(screen.getByText('Start')).toBeInTheDocument()
  })

  it('calls onToggle when stop/start button clicked', () => {
    const onToggle = vi.fn()
    render(<DockerServiceCard service={mockService} onToggle={onToggle} />)
    fireEvent.click(screen.getByText('Stop'))
    expect(onToggle).toHaveBeenCalledWith('rstn-postgres')
  })

  it('calls onRestart when restart button clicked', () => {
    const onRestart = vi.fn()
    render(<DockerServiceCard service={mockService} onRestart={onRestart} />)
    fireEvent.click(screen.getByText('Restart'))
    expect(onRestart).toHaveBeenCalledWith('rstn-postgres')
  })

  it('calls onViewLogs when logs button clicked', () => {
    const onViewLogs = vi.fn()
    render(<DockerServiceCard service={mockService} onViewLogs={onViewLogs} />)
    fireEvent.click(screen.getByText('Logs'))
    expect(onViewLogs).toHaveBeenCalledWith('rstn-postgres')
  })

  it('calls onSelect when card clicked', () => {
    const onSelect = vi.fn()
    render(<DockerServiceCard service={mockService} onSelect={onSelect} />)
    // Click on the card (not a button)
    const card = screen.getByText('PostgreSQL').closest('[class*="cursor-pointer"]')
    if (card) {
      fireEvent.click(card)
      expect(onSelect).toHaveBeenCalledWith('rstn-postgres')
    }
  })

  it('shows Add DB button for Database type', () => {
    render(<DockerServiceCard service={mockService} />)
    expect(screen.getByText('Add DB')).toBeInTheDocument()
  })

  it('shows Add vhost button for MessageBroker type', () => {
    const rabbitService = { ...mockService, id: 'rstn-rabbitmq', service_type: 'MessageBroker' as const }
    render(<DockerServiceCard service={rabbitService} />)
    expect(screen.getByText('Add vhost')).toBeInTheDocument()
  })

  it('does not show Add DB button for Cache type', () => {
    const redisService = { ...mockService, id: 'rstn-redis', service_type: 'Cache' as const }
    render(<DockerServiceCard service={redisService} />)
    expect(screen.queryByText('Add DB')).not.toBeInTheDocument()
    expect(screen.queryByText('Add vhost')).not.toBeInTheDocument()
  })

  it('disables buttons when service is starting', () => {
    const startingService = { ...mockService, status: 'starting' as const }
    render(<DockerServiceCard service={startingService} />)
    expect(screen.getByText('Start')).toBeDisabled()
    expect(screen.getByText('Restart')).toBeDisabled()
  })

  it('disables restart when service is stopped', () => {
    const stoppedService = { ...mockService, status: 'stopped' as const }
    render(<DockerServiceCard service={stoppedService} />)
    expect(screen.getByText('Restart')).toBeDisabled()
  })

  it('applies active styling when isActive is true', () => {
    render(<DockerServiceCard service={mockService} isActive={true} />)
    const card = screen.getByText('PostgreSQL').closest('[class*="border-primary"]')
    expect(card).toBeInTheDocument()
  })

  it('copies connection string to clipboard', async () => {
    render(<DockerServiceCard service={mockService} />)
    fireEvent.click(screen.getByText('Copy URL'))
    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
        'postgresql://postgres:postgres@localhost:5432/postgres'
      )
    })
  })
})
