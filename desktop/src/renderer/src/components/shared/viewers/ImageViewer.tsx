import { useState } from 'react'
import { Box, IconButton, Stack, Tooltip, Typography } from '@mui/material'
import {
  ZoomIn,
  ZoomOut,
  ZoomOutMap,
  FitScreen,
  Info as InfoIcon,
} from '@mui/icons-material'
import { formatFileSize } from '@/utils/fileTypes'

export interface ImageViewerProps {
  path: string
  size?: number
}

/**
 * Image viewer component with zoom controls
 */
export function ImageViewer({ path, size }: ImageViewerProps) {
  const [fitToWindow, setFitToWindow] = useState(true)
  const [zoom, setZoom] = useState(100)
  const [showInfo, setShowInfo] = useState(false)
  const [imageLoaded, setImageLoaded] = useState(false)
  const [imageDimensions, setImageDimensions] = useState<{ width: number; height: number } | null>(
    null
  )

  const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement>) => {
    const img = e.currentTarget
    setImageDimensions({ width: img.naturalWidth, height: img.naturalHeight })
    setImageLoaded(true)
  }

  const handleZoomIn = () => {
    setFitToWindow(false)
    setZoom((prev) => Math.min(prev + 25, 400))
  }

  const handleZoomOut = () => {
    setFitToWindow(false)
    setZoom((prev) => Math.max(prev - 25, 25))
  }

  const handleFitToWindow = () => {
    setFitToWindow(true)
    setZoom(100)
  }

  const handleActualSize = () => {
    setFitToWindow(false)
    setZoom(100)
  }

  const imageStyle: React.CSSProperties = {
    maxWidth: fitToWindow ? '100%' : 'none',
    maxHeight: fitToWindow ? 'calc(100vh - 200px)' : 'none',
    width: fitToWindow ? 'auto' : `${zoom}%`,
    height: 'auto',
    display: 'block',
    margin: '0 auto',
    objectFit: 'contain',
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Toolbar */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          px: 2,
          py: 1,
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Stack direction="row" spacing={1}>
          <Tooltip title="Zoom in">
            <IconButton size="small" onClick={handleZoomIn} disabled={zoom >= 400}>
              <ZoomIn fontSize="small" />
            </IconButton>
          </Tooltip>

          <Tooltip title="Zoom out">
            <IconButton size="small" onClick={handleZoomOut} disabled={zoom <= 25}>
              <ZoomOut fontSize="small" />
            </IconButton>
          </Tooltip>

          <Tooltip title="Actual size (100%)">
            <IconButton size="small" onClick={handleActualSize}>
              <ZoomOutMap fontSize="small" />
            </IconButton>
          </Tooltip>

          <Tooltip title="Fit to window">
            <IconButton size="small" onClick={handleFitToWindow}>
              <FitScreen fontSize="small" />
            </IconButton>
          </Tooltip>
        </Stack>

        <Typography variant="body2" sx={{ ml: 2, color: 'text.secondary' }}>
          {fitToWindow ? 'Fit' : `${zoom}%`}
        </Typography>

        <Box sx={{ flexGrow: 1 }} />

        <Tooltip title="Image info">
          <IconButton size="small" onClick={() => setShowInfo(!showInfo)}>
            <InfoIcon fontSize="small" />
          </IconButton>
        </Tooltip>
      </Box>

      {/* Info Panel */}
      {showInfo && imageDimensions && (
        <Box
          sx={{
            px: 2,
            py: 1,
            bgcolor: 'action.hover',
            borderBottom: 1,
            borderColor: 'divider',
          }}
        >
          <Stack direction="row" spacing={3}>
            <Typography variant="caption">
              <strong>Dimensions:</strong> {imageDimensions.width} × {imageDimensions.height}
            </Typography>
            {size && (
              <Typography variant="caption">
                <strong>Size:</strong> {formatFileSize(size)}
              </Typography>
            )}
            <Typography variant="caption" sx={{ color: 'text.secondary' }}>
              {path.split('/').pop()}
            </Typography>
          </Stack>
        </Box>
      )}

      {/* Image Container */}
      <Box
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          p: 2,
          bgcolor: 'background.default',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <img
          src={`file://${path}`}
          alt={path.split('/').pop()}
          style={imageStyle}
          onLoad={handleImageLoad}
          onError={(e) => {
            console.error('Failed to load image:', path, e)
          }}
        />
      </Box>

      {/* Loading indicator */}
      {!imageLoaded && (
        <Box
          sx={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
          }}
        >
          <Typography variant="body2" color="text.secondary">
            Loading image...
          </Typography>
        </Box>
      )}
    </Box>
  )
}
