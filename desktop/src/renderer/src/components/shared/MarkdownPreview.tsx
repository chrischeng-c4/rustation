import { useEffect, useId, useRef, useState } from 'react'
import { Box, CircularProgress, Stack, Typography } from '@mui/material'
import { Description } from '@mui/icons-material'
import ReactMarkdown from 'react-markdown'
import mermaid from 'mermaid'
import { MarkdownDisplay } from './MarkdownDisplay'

// Initialize mermaid with dark theme
mermaid.initialize({
  startOnLoad: false,
  theme: 'dark',
  securityLevel: 'loose',
  fontFamily: '"JetBrains Mono", "Fira Code", monospace',
})

interface MarkdownPreviewProps {
  /** Raw markdown content */
  content: string
  /** File path (for display in header) */
  path: string
  /** Whether to show the header (default: true) */
  showHeader?: boolean
}

/**
 * Mermaid diagram renderer component
 */
function MermaidBlock({ code }: { code: string }) {
  const id = useId().replace(/:/g, '_')
  const containerRef = useRef<HTMLDivElement>(null)
  const [svg, setSvg] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    async function renderDiagram() {
      try {
        const { svg: renderedSvg } = await mermaid.render(`mermaid-${id}`, code)
        if (!cancelled) {
          setSvg(renderedSvg)
          setError(null)
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to render diagram')
          setSvg(null)
        }
      }
    }

    renderDiagram()

    return () => {
      cancelled = true
    }
  }, [code, id])

  if (error) {
    return (
      <Box
        sx={{
          p: 2,
          bgcolor: 'error.main',
          color: 'error.contrastText',
          borderRadius: 1,
          fontFamily: 'monospace',
          fontSize: '0.75rem',
          my: 2,
        }}
      >
        <Typography variant="caption" fontWeight={600}>
          Mermaid Error:
        </Typography>
        <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>{error}</pre>
      </Box>
    )
  }

  if (!svg) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
        <CircularProgress size={20} />
      </Box>
    )
  }

  return (
    <Box
      ref={containerRef}
      sx={{
        my: 2,
        p: 2,
        bgcolor: 'background.paper',
        borderRadius: 1,
        border: 1,
        borderColor: 'divider',
        overflow: 'auto',
        '& svg': {
          maxWidth: '100%',
          height: 'auto',
        },
      }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  )
}

/**
 * Custom code block renderer that handles mermaid diagrams
 */
function CodeBlock({
  className,
  children,
}: {
  className?: string
  children?: React.ReactNode
}) {
  const match = /language-(\w+)/.exec(className || '')
  const language = match?.[1]
  const code = String(children).replace(/\n$/, '')

  // Render mermaid diagrams
  if (language === 'mermaid') {
    return <MermaidBlock code={code} />
  }

  // Default code block rendering
  return (
    <code className={className}>
      {children}
    </code>
  )
}

/**
 * Markdown preview component with Mermaid diagram support.
 * Used for rendering .md and .mdx files in the file explorer.
 */
export function MarkdownPreview({ content, path, showHeader = true }: MarkdownPreviewProps): React.ReactElement {
  const filename = path.split('/').pop() || ''

  // When embedded (showHeader=false), just render the content
  if (!showHeader) {
    return (
      <MarkdownDisplay>
        <ReactMarkdown
          components={{
            code: CodeBlock,
          }}
        >
          {content}
        </ReactMarkdown>
      </MarkdownDisplay>
    )
  }

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        minHeight: 0,
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <Stack
        direction="row"
        alignItems="center"
        spacing={1}
        sx={{
          px: 2,
          py: 0.5,
          borderBottom: 1,
          borderColor: 'divider',
          flexShrink: 0,
        }}
      >
        <Description fontSize="small" />
        <Typography variant="caption" sx={{ fontFamily: 'monospace', flex: 1 }}>
          {filename}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          Markdown
        </Typography>
      </Stack>

      {/* Content */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        <MarkdownDisplay>
          <ReactMarkdown
            components={{
              code: CodeBlock,
            }}
          >
            {content}
          </ReactMarkdown>
        </MarkdownDisplay>
      </Box>
    </Box>
  )
}

export default MarkdownPreview
