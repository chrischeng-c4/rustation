import React from 'react'
import { Box, Button, Stack, Typography } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

interface EmptyStateProps {
  icon?: React.ReactNode // Changed to ReactNode to allow passing icon with props
  title: string
  description: string
  action?: {
    label: string
    onClick: () => void
    icon?: React.ReactNode
  }
  sx?: SxProps<Theme>
}

/**
 * EmptyState - Standardized placeholder for empty views or lists.
 */
export function EmptyState({
  icon,
  title,
  description,
  action,
  sx,
}: EmptyStateProps) {
  return (
    <Stack
      alignItems="center"
      justifyContent="center"
      spacing={3}
      sx={{ height: '100%', p: 4, textAlign: 'center', ...sx }}
    >
      {icon && (
        <Box
          sx={{
            height: 80,
            width: 80,
            borderRadius: 4, // M3 Rounded square
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'secondaryContainer.main',
            color: 'onSecondaryContainer.main',
            '& .MuiSvgIcon-root': { fontSize: 40 }
          }}
        >
          {icon}
        </Box>
      )}
      
      <Box>
        <Typography variant="h6" fontWeight={600}>
          {title}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ maxWidth: 280, mt: 1 }}>
          {description}
        </Typography>
      </Box>

      {action && (
        <Button
          variant="contained"
          onClick={action.onClick}
          startIcon={action.icon}
          sx={{ borderRadius: 2 }}
        >
          {action.label}
        </Button>
      )}
    </Stack>
  )
}
