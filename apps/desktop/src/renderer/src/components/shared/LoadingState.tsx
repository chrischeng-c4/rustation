import { CircularProgress, Stack, Typography } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

interface LoadingStateProps {
  message?: string
  sx?: SxProps<Theme>
}

/**
 * LoadingState - Standardized loading indicator for pages or sections.
 */
export function LoadingState({
  message = "Loading...",
  sx,
}: LoadingStateProps) {
  return (
    <Stack
      sx={{
        height: '100%',
        width: '100%',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
        ...sx,
      }}
    >
      <CircularProgress size={32} />
      {message && (
        <Typography variant="body2" color="text.secondary" fontWeight={500}>
          {message}
        </Typography>
      )}
    </Stack>
  )
}
