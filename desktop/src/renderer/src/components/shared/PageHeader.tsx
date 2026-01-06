import React from 'react'
import { Box, Stack, Typography } from '@mui/material'
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
    <Box sx={{ mb: 4, ...sx }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
        <Stack direction="row" alignItems="center" spacing={2}>
          {icon && (
            <Box 
              sx={{ 
                color: 'primary.main',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                '& .MuiSvgIcon-root': { fontSize: 28 }
              }}
            >
              {icon}
            </Box>
          )}
          <Box>
            <Typography variant="h5" fontWeight={600} sx={{ letterSpacing: '-0.01em' }}>
              {title}
            </Typography>
            {description && (
              <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
                {description}
              </Typography>
            )}
          </Box>
        </Stack>
        
        <Stack direction="row" alignItems="center" spacing={1.5}>
          {children}
        </Stack>
      </Stack>
    </Box>
  )
}
