import React from 'react'
import { Box, Divider, Stack, Typography } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

interface PageHeaderProps {
  title: string
  description?: string
  children?: React.ReactNode
  sx?: SxProps<Theme>
  icon?: React.ReactNode
}

/**
 * PageHeader - Standardized header for feature pages.
 * Includes title, description, and an action area for buttons.
 */
export function PageHeader({
  title,
  description,
  children,
  sx,
  icon,
}: PageHeaderProps) {
  return (
    <Stack spacing={2} sx={{ mb: 3, ...sx }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between">
        <Box>
          <Stack direction="row" alignItems="center" spacing={1}>
            {icon && <Box sx={{ color: 'text.secondary' }}>{icon}</Box>}
            <Typography variant="h5" fontWeight={600}>
              {title}
            </Typography>
          </Stack>
          {description && (
            <Typography variant="body2" color="text.secondary">
              {description}
            </Typography>
          )}
        </Box>
        <Stack direction="row" alignItems="center" spacing={1}>
          {children}
        </Stack>
      </Stack>
      <Divider />
    </Stack>
  )
}
