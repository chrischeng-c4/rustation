import { useState } from 'react'
import { Box, Button, Stack, Typography } from '@mui/material'
import { PlayArrow } from '@mui/icons-material'
import { formatFileSize } from '@/utils/fileTypes'

export interface VideoViewerProps {
  path: string
  size?: number
}

/**
 * Video viewer component with lazy loading
 */
export function VideoViewer({ path, size }: VideoViewerProps) {
  const [loaded, setLoaded] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleLoad = () => {
    setLoaded(true)
  }

  const handleError = () => {
    setError('Failed to load video')
  }

  const filename = path.split('/').pop() || 'video'

  // Show preview/thumbnail before loading
  if (!loaded) {
    return (
      <Box
        sx={{
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'background.default',
          p: 4,
        }}
      >
        <Stack spacing={2} alignItems="center">
          <PlayArrow sx={{ fontSize: 64, color: 'text.secondary' }} />
          <Typography variant="h6" color="text.secondary">
            {filename}
          </Typography>
          {size && (
            <Typography variant="body2" color="text.secondary">
              {formatFileSize(size)}
            </Typography>
          )}
          <Button variant="contained" onClick={handleLoad} startIcon={<PlayArrow />}>
            Load Video
          </Button>
        </Stack>
      </Box>
    )
  }

  return (
    <Box
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'background.default',
      }}
    >
      {/* Video Info Bar */}
      <Box
        sx={{
          px: 2,
          py: 1,
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Stack direction="row" spacing={2}>
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            {filename}
          </Typography>
          {size && (
            <Typography variant="body2" sx={{ color: 'text.secondary' }}>
              {formatFileSize(size)}
            </Typography>
          )}
        </Stack>
      </Box>

      {/* Video Player */}
      <Box
        sx={{
          flexGrow: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          p: 2,
        }}
      >
        {error ? (
          <Typography variant="body1" color="error">
            {error}
          </Typography>
        ) : (
          <video
            controls
            style={{
              maxWidth: '100%',
              maxHeight: '100%',
              width: 'auto',
              height: 'auto',
            }}
            onError={handleError}
          >
            <source src={`file://${path}`} />
            Your browser does not support the video tag.
          </video>
        )}
      </Box>
    </Box>
  )
}
