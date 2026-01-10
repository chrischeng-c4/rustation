import { useState, useEffect } from 'react'
import { Document, Page, pdfjs } from 'react-pdf'
import { Box, IconButton, Stack, Typography, TextField } from '@mui/material'
import {
  ChevronLeft,
  ChevronRight,
  ZoomIn,
  ZoomOut,
  Fullscreen,
} from '@mui/icons-material'

// Configure PDF.js worker
pdfjs.GlobalWorkerOptions.workerSrc = `//unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`

export interface PdfViewerProps {
  binaryContent: Uint8Array
  path: string
}

/**
 * PDF viewer with pagination and zoom controls
 */
export function PdfViewer({ binaryContent, path }: PdfViewerProps) {
  const [numPages, setNumPages] = useState<number>(0)
  const [pageNumber, setPageNumber] = useState<number>(1)
  const [scale, setScale] = useState<number>(1.0)
  const [pageInput, setPageInput] = useState<string>('1')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setPageInput(pageNumber.toString())
  }, [pageNumber])

  const onDocumentLoadSuccess = ({ numPages }: { numPages: number }) => {
    setNumPages(numPages)
    setError(null)
  }

  const onDocumentLoadError = (error: Error) => {
    console.error('PDF load error:', error)
    setError('Failed to load PDF')
  }

  const handlePrevPage = () => {
    setPageNumber((prev) => Math.max(prev - 1, 1))
  }

  const handleNextPage = () => {
    setPageNumber((prev) => Math.min(prev + 1, numPages))
  }

  const handleZoomIn = () => {
    setScale((prev) => Math.min(prev + 0.25, 3.0))
  }

  const handleZoomOut = () => {
    setScale((prev) => Math.max(prev - 0.25, 0.5))
  }

  const handlePageInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPageInput(e.target.value)
  }

  const handlePageInputBlur = () => {
    const page = parseInt(pageInput, 10)
    if (!isNaN(page) && page >= 1 && page <= numPages) {
      setPageNumber(page)
    } else {
      setPageInput(pageNumber.toString())
    }
  }

  const handlePageInputKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handlePageInputBlur()
    }
  }

  const filename = path.split('/').pop() || 'document.pdf'

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Toolbar */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 2,
          px: 2,
          py: 1,
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Typography variant="body2" sx={{ color: 'text.secondary', flexShrink: 0 }}>
          {filename}
        </Typography>

        <Box sx={{ flexGrow: 1 }} />

        {/* Pagination Controls */}
        <Stack direction="row" spacing={1} alignItems="center">
          <IconButton size="small" onClick={handlePrevPage} disabled={pageNumber <= 1}>
            <ChevronLeft />
          </IconButton>

          <TextField
            size="small"
            value={pageInput}
            onChange={handlePageInputChange}
            onBlur={handlePageInputBlur}
            onKeyDown={handlePageInputKeyDown}
            sx={{
              width: 60,
              '& .MuiInputBase-input': {
                textAlign: 'center',
                py: 0.5,
                fontSize: '0.875rem',
              },
            }}
          />

          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            / {numPages || '-'}
          </Typography>

          <IconButton size="small" onClick={handleNextPage} disabled={pageNumber >= numPages}>
            <ChevronRight />
          </IconButton>
        </Stack>

        {/* Zoom Controls */}
        <Stack direction="row" spacing={1}>
          <IconButton size="small" onClick={handleZoomOut} disabled={scale <= 0.5}>
            <ZoomOut fontSize="small" />
          </IconButton>

          <Typography variant="body2" sx={{ color: 'text.secondary', minWidth: 50, textAlign: 'center' }}>
            {Math.round(scale * 100)}%
          </Typography>

          <IconButton size="small" onClick={handleZoomIn} disabled={scale >= 3.0}>
            <ZoomIn fontSize="small" />
          </IconButton>

          <IconButton size="small" title="Fullscreen">
            <Fullscreen fontSize="small" />
          </IconButton>
        </Stack>
      </Box>

      {/* PDF Content */}
      <Box
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          bgcolor: 'background.default',
          display: 'flex',
          justifyContent: 'center',
          p: 2,
        }}
      >
        {error ? (
          <Typography variant="body1" color="error">
            {error}
          </Typography>
        ) : (
          <Document
            file={{ data: binaryContent }}
            onLoadSuccess={onDocumentLoadSuccess}
            onLoadError={onDocumentLoadError}
            loading={
              <Typography variant="body2" color="text.secondary">
                Loading PDF...
              </Typography>
            }
          >
            <Page
              pageNumber={pageNumber}
              scale={scale}
              loading={
                <Typography variant="body2" color="text.secondary">
                  Loading page...
                </Typography>
              }
            />
          </Document>
        )}
      </Box>
    </Box>
  )
}
