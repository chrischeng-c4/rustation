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
 * Smaller and more compact than PageHeader, designed for nested views.
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
        borderBottom: 1,
        borderColor: 'divider',
        bgcolor: 'background.paper',
        px: 2,
        py: 1,
        height: 48,
        flexShrink: 0,
        ...sx,
      }}
    >
      <Stack direction="row" alignItems="center" spacing={1.5} sx={{ minWidth: 0 }}>
        {icon && <Box sx={{ display: 'flex', flexShrink: 0 }}>{icon}</Box>}
        <Stack spacing={0.25} sx={{ minWidth: 0 }}>
          <Stack direction="row" alignItems="center" spacing={1}>
            <Typography variant="subtitle2" noWrap>
              {title}
            </Typography>
            {status && (
              <Chip
                label={status}
                size="small"
                color={statusColor}
                sx={{ height: 18, fontSize: '0.625rem', fontWeight: 600 }}
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
