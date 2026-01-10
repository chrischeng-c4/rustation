import { useState, useEffect } from 'react'
import { Box, Typography, CircularProgress } from '@mui/material'
import mammoth from 'mammoth'

export interface WordViewerProps {
  binaryContent: Uint8Array
  path: string
}

/**
 * Word document viewer (.docx)
 * Converts Word documents to HTML for preview
 */
export function WordViewer({ binaryContent, path }: WordViewerProps) {
  const [html, setHtml] = useState<string>('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const convertToHtml = async () => {
      try {
        setLoading(true)
        setError(null)

        const result = await mammoth.convertToHtml(
          { arrayBuffer: binaryContent.buffer as ArrayBuffer },
          {
            styleMap: [
              "p[style-name='Heading 1'] => h1",
              "p[style-name='Heading 2'] => h2",
              "p[style-name='Heading 3'] => h3",
              "p[style-name='Code'] => pre",
            ],
          }
        )

        setHtml(result.value)

        // Log any conversion warnings
        if (result.messages.length > 0) {
          console.warn('Word conversion warnings:', result.messages)
        }
      } catch (err) {
        console.error('Failed to convert Word document:', err)
        setError(err instanceof Error ? err.message : 'Failed to convert document')
      } finally {
        setLoading(false)
      }
    }

    convertToHtml()
  }, [binaryContent])

  const filename = path.split('/').pop() || 'document.docx'

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box
        sx={{
          px: 2,
          py: 1,
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Typography variant="body2" sx={{ color: 'text.secondary' }}>
          {filename}
        </Typography>
        {error && (
          <Typography variant="caption" color="error" sx={{ display: 'block', mt: 0.5 }}>
            Note: Some formatting may not be preserved
          </Typography>
        )}
      </Box>

      {/* Content */}
      <Box
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          p: 3,
          bgcolor: 'background.default',
        }}
      >
        {loading ? (
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              gap: 2,
            }}
          >
            <CircularProgress size={40} />
            <Typography variant="body2" color="text.secondary">
              Converting document...
            </Typography>
          </Box>
        ) : error ? (
          <Box sx={{ textAlign: 'center', py: 4 }}>
            <Typography variant="body1" color="error" gutterBottom>
              Failed to load document
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {error}
            </Typography>
          </Box>
        ) : (
          <Box
            sx={{
              maxWidth: 800,
              margin: '0 auto',
              bgcolor: 'background.paper',
              p: 4,
              borderRadius: 1,
              boxShadow: 1,
              // Style the converted HTML
              '& h1': {
                fontSize: '2rem',
                fontWeight: 600,
                mt: 3,
                mb: 2,
              },
              '& h2': {
                fontSize: '1.5rem',
                fontWeight: 600,
                mt: 2.5,
                mb: 1.5,
              },
              '& h3': {
                fontSize: '1.25rem',
                fontWeight: 600,
                mt: 2,
                mb: 1,
              },
              '& p': {
                mb: 1.5,
                lineHeight: 1.6,
              },
              '& ul, & ol': {
                mb: 1.5,
                pl: 3,
              },
              '& li': {
                mb: 0.5,
              },
              '& pre': {
                bgcolor: 'action.hover',
                p: 2,
                borderRadius: 1,
                overflow: 'auto',
                mb: 1.5,
              },
              '& table': {
                borderCollapse: 'collapse',
                width: '100%',
                mb: 1.5,
              },
              '& th, & td': {
                border: '1px solid',
                borderColor: 'divider',
                px: 1.5,
                py: 1,
              },
              '& th': {
                bgcolor: 'action.hover',
                fontWeight: 600,
              },
              '& img': {
                maxWidth: '100%',
                height: 'auto',
                display: 'block',
                my: 2,
              },
            }}
            dangerouslySetInnerHTML={{ __html: html }}
          />
        )}
      </Box>
    </Box>
  )
}
