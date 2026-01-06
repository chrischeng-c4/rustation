import { Alert, Typography } from '@mui/material'
import { ErrorOutline } from '@mui/icons-material'
import type { SxProps, Theme } from '@mui/material/styles'

interface ErrorBannerProps {
  error: string
  sx?: SxProps<Theme>
}

/**
 * ErrorBanner - Standardized error display for features.
 */
export function ErrorBanner({ error, sx }: ErrorBannerProps) {
  return (
    <Alert
      severity="error"
      icon={<ErrorOutline fontSize="small" />}
      sx={{ mb: 2, ...sx }}
    >
      <Typography variant="body2">{error}</Typography>
    </Alert>
  )
}
