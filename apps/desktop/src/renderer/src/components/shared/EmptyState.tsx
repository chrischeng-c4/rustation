import React from 'react'
import { Box, Button, Stack, Typography } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

interface EmptyStateProps {
  icon: React.ElementType
  title: string
  description: string
  action?: {
    label: string
    onClick: () => void
    icon?: React.ElementType
  }
  sx?: SxProps<Theme>
}

/**
 * EmptyState - Standardized placeholder for empty views or lists.
 */
export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  sx,
}: EmptyStateProps) {
  return (
    <Stack
      alignItems="center"
      justifyContent="center"
      spacing={2}
      sx={{ height: '100%', p: 4, textAlign: 'center', ...sx }}
    >
      <Box
        sx={{
          height: 80,
          width: 80,
          borderRadius: '50%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'action.hover',
        }}
      >
        <Icon sx={{ fontSize: 40, color: 'text.secondary' }} />
      </Box>
      <Typography variant="h6" fontWeight={600}>
        {title}
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ maxWidth: 250 }}>
        {description}
      </Typography>
      {action && (
        <Button
          variant="outlined"
          onClick={action.onClick}
          startIcon={action.icon ? <action.icon /> : undefined}
        >
          {action.label}
        </Button>
      )}
    </Stack>
  )
}
