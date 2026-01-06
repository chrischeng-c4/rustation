import React from 'react'
import { Box, Chip, Stack, Typography } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

interface WorkflowHeaderProps {
  title: string
  subtitle?: string
  status?: string
  statusColor?: 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info'
  icon?: React.ReactNode
  children?: React.ReactNode
  sx?: SxProps<Theme>
}

/**
 * WorkflowHeader - Standardized header for workflow sub-panels.
 */
export function WorkflowHeader({
  title,
  subtitle,
  status,
  statusColor = "info",
  icon,
  children,
  sx,
}: WorkflowHeaderProps) {
  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      sx={{
        bgcolor: 'surfaceContainerLow.main',
        px: 3,
        py: 1.5,
        height: 64,
        flexShrink: 0,
        ...sx,
      }}
    >
      <Stack direction="row" alignItems="center" spacing={2} sx={{ minWidth: 0 }}>
        {icon && <Box sx={{ display: 'flex', flexShrink: 0, color: 'primary.main' }}>{icon}</Box>}
        <Stack spacing={0} sx={{ minWidth: 0 }}>
          <Stack direction="row" alignItems="center" spacing={1}>
            <Typography variant="subtitle1" fontWeight={600} noWrap>
              {title}
            </Typography>
            {status && (
              <Chip
                label={status}
                size="small"
                color={statusColor}
                sx={{ height: 20, fontSize: '0.65rem', fontWeight: 700, borderRadius: 1 }}
              />
            )}
          </Stack>
          {subtitle && (
            <Typography variant="caption" color="text.secondary" noWrap>
              {subtitle}
            </Typography>
          )}
        </Stack>
      </Stack>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ flexShrink: 0 }}>
        {children}
      </Stack>
    </Stack>
  )
}
