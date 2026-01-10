import * as React from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  Alert,
  Box,
  CircularProgress,
  IconButton,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import { AddComment, Cancel, Code, Description, ErrorOutline, Send, Visibility } from '@mui/icons-material'
import { Highlight, themes } from 'prism-react-renderer'
import Prism from 'prismjs'
import 'prismjs/components/prism-yaml'
import { List, RowComponentProps, useListRef } from 'react-window'
import { AutoSizer } from 'react-virtualized-auto-sizer'
import { useAppState } from '@/hooks/useAppState'
import { MarkdownPreview } from './MarkdownPreview'
import { getFileCategory, isBinaryFile } from '@/utils/fileTypes'
import { ImageViewer } from './viewers/ImageViewer'
import { VideoViewer } from './viewers/VideoViewer'
import { PdfViewer } from './viewers/PdfViewer'
import { WordViewer } from './viewers/WordViewer'
import { CsvViewer } from './viewers/CsvViewer'
import { ExcelViewer } from './viewers/ExcelViewer'

/** Comment data structure matching backend */
export interface CommentData {
  id: string
  content: string
  author: string
  created_at: string
  line_number: number | null
}

interface SourceCodeViewerProps {
  /** Absolute or relative path to the file */
  path: string
  /** Project root for security validation */
  projectRoot: string
  /** Optional: Show line numbers (default: true) */
  showLineNumbers?: boolean
  /** Optional: Maximum height with scroll */
  maxHeight?: string
  /** Optional: Callback when file cannot be read */
  onError?: (error: string) => void
  /** Optional: Comments to display inline */
  comments?: CommentData[]
  /** Optional: Callback to add a new inline comment */
  onAddComment?: (lineNumber: number, content: string) => void | Promise<void>
}

/** Threshold for enabling virtualization (lines) */
const VIRTUALIZATION_THRESHOLD = 500
/** Line height in pixels */
const LINE_HEIGHT = 20

/**
 * Check if file is a markdown file that should be rendered
 */
function isMarkdownFile(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() || ''
  return ext === 'md' || ext === 'mdx'
}

/**
 * Parse error code from Rust error message format "CODE: message"
 */
function parseErrorMessage(error: string): { code: string; message: string } {
  const colonIndex = error.indexOf(':')
  if (colonIndex > 0) {
    const code = error.substring(0, colonIndex).trim()
    const message = error.substring(colonIndex + 1).trim()
    return { code, message }
  }
  return { code: 'UNKNOWN', message: error }
}

/**
 * Get user-friendly error message based on error code
 */
function getFriendlyErrorMessage(code: string, path: string): string {
  switch (code) {
    case 'FILE_NOT_FOUND':
      return `File not found: ${path}`
    case 'SECURITY_VIOLATION':
      return 'Access denied: File is outside project scope'
    case 'FILE_TOO_LARGE':
      return 'File too large to display (max 10MB)'
    case 'NOT_UTF8':
      return 'Cannot display: File is not UTF-8 text'
    case 'PERMISSION_DENIED':
      return 'Permission denied: Cannot read file'
    default:
      return `Error reading file: ${path}`
  }
}

/**
 * Get language from file extension for syntax highlighting
 */
function getLanguageFromPath(path: string): string {
  const filename = path.split('/').pop()?.toLowerCase() || ''
  const ext = filename.split('.').pop()?.toLowerCase() || ''

  // Handle special filenames without extensions
  const filenameMap: Record<string, string> = {
    dockerfile: 'docker',
    makefile: 'makefile',
    justfile: 'makefile',
    gemfile: 'ruby',
    rakefile: 'ruby',
    podfile: 'ruby',
    vagrantfile: 'ruby',
    '.gitignore': 'git',
    '.gitattributes': 'git',
    '.dockerignore': 'docker',
    '.env': 'bash',
    '.env.local': 'bash',
    '.env.development': 'bash',
    '.env.production': 'bash',
    '.bashrc': 'bash',
    '.zshrc': 'bash',
    '.profile': 'bash',
  }

  if (filenameMap[filename]) {
    return filenameMap[filename]
  }

  const languageMap: Record<string, string> = {
    // Rust
    rs: 'rust',
    // JavaScript/TypeScript
    js: 'javascript',
    jsx: 'jsx',
    ts: 'typescript',
    tsx: 'tsx',
    mjs: 'javascript',
    cjs: 'javascript',
    // Web
    html: 'markup',
    htm: 'markup',
    xml: 'markup',
    svg: 'markup',
    vue: 'markup',
    svelte: 'markup',
    // Styles
    css: 'css',
    scss: 'scss',
    sass: 'sass',
    less: 'less',
    styl: 'stylus',
    // Data formats
    json: 'json',
    json5: 'json5',
    yaml: 'yaml',
    yml: 'yaml',
    toml: 'toml',
    ini: 'ini',
    // Shell
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    fish: 'bash',
    ps1: 'powershell',
    psm1: 'powershell',
    bat: 'batch',
    cmd: 'batch',
    // Python
    py: 'python',
    pyw: 'python',
    pyx: 'python',
    pyi: 'python',
    // Go
    go: 'go',
    mod: 'go',
    // C/C++
    c: 'c',
    h: 'c',
    cpp: 'cpp',
    hpp: 'cpp',
    cc: 'cpp',
    cxx: 'cpp',
    hxx: 'cpp',
    // C#
    cs: 'csharp',
    csx: 'csharp',
    // Java/Kotlin/Scala
    java: 'java',
    kt: 'kotlin',
    kts: 'kotlin',
    scala: 'scala',
    sc: 'scala',
    // Swift/Objective-C
    swift: 'swift',
    m: 'objectivec',
    mm: 'objectivec',
    // Ruby
    rb: 'ruby',
    erb: 'erb',
    rake: 'ruby',
    // PHP
    php: 'php',
    phtml: 'php',
    // Lua
    lua: 'lua',
    // Perl
    pl: 'perl',
    pm: 'perl',
    // R
    r: 'r',
    rmd: 'r',
    // SQL
    sql: 'sql',
    // Markdown
    md: 'markdown',
    mdx: 'markdown',
    // Docker
    dockerfile: 'docker',
    // GraphQL
    graphql: 'graphql',
    gql: 'graphql',
    // Diff/Patch
    diff: 'diff',
    patch: 'diff',
    // Elixir/Erlang
    ex: 'elixir',
    exs: 'elixir',
    erl: 'erlang',
    hrl: 'erlang',
    // Haskell
    hs: 'haskell',
    lhs: 'haskell',
    // Clojure
    clj: 'clojure',
    cljs: 'clojure',
    cljc: 'clojure',
    edn: 'clojure',
    // F#
    fs: 'fsharp',
    fsi: 'fsharp',
    fsx: 'fsharp',
    // OCaml
    ml: 'ocaml',
    mli: 'ocaml',
    // Zig
    zig: 'zig',
    // Nim
    nim: 'nim',
    // V
    v: 'v',
    // Dart
    dart: 'dart',
    // Julia
    jl: 'julia',
    // Terraform
    tf: 'hcl',
    tfvars: 'hcl',
    // Nix
    nix: 'nix',
    // Prisma
    prisma: 'prisma',
    // Protocol Buffers
    proto: 'protobuf',
    // Assembly
    asm: 'asm6502',
    s: 'asm6502',
    // WASM
    wat: 'wasm',
    wast: 'wasm',
    // LaTeX
    tex: 'latex',
    sty: 'latex',
    // Regex
    regex: 'regex',
  }
  return languageMap[ext] || 'text'
}

/**
 * Inline comment input component
 */
function InlineCommentInput({
  lineNumber,
  onSubmit,
  onCancel,
}: {
  lineNumber: number
  onSubmit: (content: string) => void
  onCancel: () => void
}): React.ReactElement {
  const [content, setContent] = useState('')

  const handleSubmit = useCallback(() => {
    if (content.trim()) {
      onSubmit(content.trim())
      setContent('')
    }
  }, [content, onSubmit])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        handleSubmit()
      } else if (e.key === 'Escape') {
        onCancel()
      }
    },
    [handleSubmit, onCancel]
  )

  return (
    <Box
      sx={{
        display: 'flex',
        gap: 1,
        alignItems: 'flex-start',
        p: 1,
        ml: 6,
        bgcolor: 'action.hover',
        borderRadius: 1,
        border: 1,
        borderColor: 'primary.main',
      }}
    >
      <TextField
        size="small"
        placeholder={`Add comment on line ${lineNumber}...`}
        value={content}
        onChange={(e) => setContent(e.target.value)}
        onKeyDown={handleKeyDown}
        multiline
        maxRows={4}
        autoFocus
        sx={{ flex: 1, '& .MuiInputBase-input': { fontSize: '0.75rem' } }}
      />
      <IconButton size="small" onClick={handleSubmit} disabled={!content.trim()} color="primary">
        <Send fontSize="small" />
      </IconButton>
      <IconButton size="small" onClick={onCancel}>
        <Cancel fontSize="small" />
      </IconButton>
    </Box>
  )
}

/**
 * Inline comment display component
 */
function InlineComment({ comment }: { comment: CommentData }): React.ReactElement {
  const timestamp = new Date(comment.created_at).toLocaleString()
  return (
    <Box
      sx={{
        display: 'flex',
        gap: 1,
        alignItems: 'flex-start',
        p: 0.5,
        pl: 7,
        bgcolor: 'action.selected',
        borderLeft: 3,
        borderColor: 'info.main',
        fontSize: '0.75rem',
      }}
    >
      <Box sx={{ flex: 1 }}>
        <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem' }}>
          {comment.author} · {timestamp}
        </Typography>
        <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap', fontSize: '0.75rem' }}>
          {comment.content}
        </Typography>
      </Box>
    </Box>
  )
}

/** Token type from prism-react-renderer */
type Token = { types: string[]; content: string; empty?: boolean }

/** Data passed to virtualized row renderer */
interface RowData {
  tokens: Token[][]
  showLineNumbers: boolean
  onAddComment?: (lineNumber: number, content: string) => void
  commentsByLine: Map<number, CommentData[]>
  addingCommentLine: number | null
  onLineClick: (lineNumber: number) => void
  onCommentSubmit: (content: string) => void
  onCommentCancel: () => void
  getTokenStyle: (token: Token) => React.CSSProperties
}

/**
 * Virtualized row component for react-window v2
 */
function VirtualizedRow({ index, style, ...props }: RowComponentProps<RowData>) {
  const {
    tokens,
    showLineNumbers,
    onAddComment,
    commentsByLine,
    addingCommentLine,
    onLineClick,
    onCommentSubmit,
    onCommentCancel,
    getTokenStyle,
  } = props as RowData

  const line = tokens[index]
  const lineNumber = index + 1
  const lineComments = commentsByLine.get(lineNumber) || []
  const isAddingHere = addingCommentLine === lineNumber

  return (
    <div style={style}>
      <Box
        sx={{
          display: 'flex',
          height: LINE_HEIGHT,
          '&:hover': {
            bgcolor: 'action.hover',
            '& .add-comment-btn': { opacity: 1 },
          },
        }}
      >
        {showLineNumbers && (
          <Box
            component="span"
            sx={{
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'flex-end',
              gap: 0.5,
              width: 56,
              minWidth: 56,
              pr: 1,
              pl: 0.5,
              color: 'text.secondary',
              userSelect: 'none',
              fontSize: '0.75rem',
              borderRight: 1,
              borderColor: 'divider',
              bgcolor: 'background.paper',
            }}
          >
            {onAddComment && (
              <Tooltip title="Add comment" placement="left">
                <IconButton
                  size="small"
                  className="add-comment-btn"
                  onClick={() => onLineClick(lineNumber)}
                  sx={{
                    opacity: 0,
                    p: 0.25,
                    transition: 'opacity 0.15s',
                  }}
                >
                  <AddComment sx={{ fontSize: 12 }} />
                </IconButton>
              </Tooltip>
            )}
            {lineNumber}
          </Box>
        )}
        <Box
          component="span"
          sx={{
            flex: 1,
            pl: 1.5,
            pr: 1,
            whiteSpace: 'pre',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
          }}
        >
          {line.map((token, tokenIndex) => (
            <span key={tokenIndex} style={getTokenStyle(token)}>
              {token.content}
            </span>
          ))}
        </Box>
      </Box>
      {/* Render inline comments */}
      {lineComments.map((comment) => (
        <InlineComment key={comment.id} comment={comment} />
      ))}
      {/* Render comment input */}
      {isAddingHere && (
        <InlineCommentInput lineNumber={lineNumber} onSubmit={onCommentSubmit} onCancel={onCommentCancel} />
      )}
    </div>
  )
}

/**
 * Non-virtualized renderer for small files
 */
function SimpleCodeRenderer({
  tokens,
  showLineNumbers,
  onAddComment,
  commentsByLine,
  addingCommentLine,
  onLineClick,
  onCommentSubmit,
  onCommentCancel,
  getTokenStyle,
}: RowData): React.ReactElement {
  return (
    <Box component="pre" sx={{ m: 0, p: 0 }}>
      {tokens.map((line, lineIndex) => {
        const lineNumber = lineIndex + 1
        const lineComments = commentsByLine.get(lineNumber) || []
        const isAddingHere = addingCommentLine === lineNumber

        return (
          <React.Fragment key={lineIndex}>
            <Box
              sx={{
                display: 'flex',
                minHeight: LINE_HEIGHT,
                '&:hover': {
                  bgcolor: 'action.hover',
                  '& .add-comment-btn': { opacity: 1 },
                },
              }}
            >
              {showLineNumbers && (
                <Box
                  component="span"
                  sx={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    justifyContent: 'flex-end',
                    gap: 0.5,
                    width: 56,
                    minWidth: 56,
                    pr: 1,
                    pl: 0.5,
                    color: 'text.secondary',
                    userSelect: 'none',
                    fontSize: '0.75rem',
                    borderRight: 1,
                    borderColor: 'divider',
                    position: 'sticky',
                    left: 0,
                    bgcolor: 'background.paper',
                    zIndex: 1,
                  }}
                >
                  {onAddComment && (
                    <Tooltip title="Add comment" placement="left">
                      <IconButton
                        size="small"
                        className="add-comment-btn"
                        onClick={() => onLineClick(lineNumber)}
                        sx={{
                          opacity: 0,
                          p: 0.25,
                          transition: 'opacity 0.15s',
                        }}
                      >
                        <AddComment sx={{ fontSize: 12 }} />
                      </IconButton>
                    </Tooltip>
                  )}
                  {lineNumber}
                </Box>
              )}
              <Box
                component="span"
                sx={{
                  flex: 1,
                  pl: 1.5,
                  pr: 1,
                  whiteSpace: 'pre',
                }}
              >
                {line.map((token, tokenIndex) => (
                  <span key={tokenIndex} style={getTokenStyle(token)}>
                    {token.content}
                  </span>
                ))}
              </Box>
            </Box>
            {/* Render inline comments */}
            {lineComments.map((comment) => (
              <InlineComment key={comment.id} comment={comment} />
            ))}
            {/* Render comment input */}
            {isAddingHere && (
              <InlineCommentInput lineNumber={lineNumber} onSubmit={onCommentSubmit} onCancel={onCommentCancel} />
            )}
          </React.Fragment>
        )
      })}
    </Box>
  )
}

/** Child component for AutoSizer */
function AutoSizerChild({
  width,
  height,
  rowData,
}: {
  width: number
  height: number
  rowData: RowData
}) {
  const listRef = useListRef(null)

  if (width === 0 || height === 0) return null

  return (
    <List<RowData>
      listRef={listRef}
      style={{ width, height }}
      rowCount={rowData.tokens.length}
      rowHeight={LINE_HEIGHT}
      rowComponent={VirtualizedRow}
      rowProps={rowData}
      overscanCount={20}
    />
  )
}

/** Virtualized List wrapper that uses AutoSizer */
function VirtualizedList({ rowData }: { rowData: RowData }) {
  return (
    <AutoSizer
      renderProp={({ width, height }) => (
        <AutoSizerChild width={width ?? 0} height={height ?? 0} rowData={rowData} />
      )}
    />
  )
}

/**
 * Component for viewing source code files with syntax highlighting.
 * Uses virtualization for large files (>500 lines).
 */
export function SourceCodeViewer({
  path,
  projectRoot: _projectRoot,
  showLineNumbers = true,
  maxHeight: _maxHeight,
  onError,
  comments = [],
  onAddComment,
}: SourceCodeViewerProps): React.ReactElement {
  const { state: appState, dispatch } = useAppState()
  const viewerState = appState?.file_viewer
  const [addingCommentLine, setAddingCommentLine] = useState<number | null>(null)
  // For markdown files: 'preview' = rendered, 'source' = syntax highlighted with inline comments
  const [markdownViewMode, setMarkdownViewMode] = useState<'preview' | 'source'>('preview')

  // Detect file category and dispatch appropriate read action
  const fileCategory = useMemo(() => getFileCategory(path), [path])
  const needsBinary = useMemo(() => isBinaryFile(fileCategory), [fileCategory])

  useEffect(() => {
    if (needsBinary) {
      dispatch({ type: 'ReadBinaryFile', payload: { path } })
    } else {
      dispatch({ type: 'ReadFile', payload: { path } })
    }
  }, [path, needsBinary, dispatch])

  // Handle errors when they occur in the global state
  useEffect(() => {
    if (viewerState?.error && viewerState.path === path) {
      const { code } = parseErrorMessage(viewerState.error)
      const friendlyMessage = getFriendlyErrorMessage(code, path)
      onError?.(friendlyMessage)
    }
  }, [viewerState?.error, viewerState?.path, path, onError])

  // Group comments by line number for efficient lookup
  const commentsByLine = useMemo(() => {
    console.log('[SourceCodeViewer] Grouping comments:', comments)
    const map = new Map<number, CommentData[]>()
    for (const comment of comments) {
      if (comment.line_number !== null) {
        const existing = map.get(comment.line_number) || []
        existing.push(comment)
        map.set(comment.line_number, existing)
      }
    }
    console.log('[SourceCodeViewer] Comments by line:', Array.from(map.entries()))
    return map
  }, [comments])

  const handleLineClick = useCallback(
    (lineNumber: number) => {
      if (onAddComment) {
        setAddingCommentLine(lineNumber)
      }
    },
    [onAddComment]
  )

  const handleCommentSubmit = useCallback(
    async (content: string) => {
      if (addingCommentLine !== null && onAddComment) {
        await onAddComment(addingCommentLine, content)
        setAddingCommentLine(null)
      }
    },
    [addingCommentLine, onAddComment]
  )

  const handleCommentCancel = useCallback(() => {
    setAddingCommentLine(null)
  }, [])

  if (!viewerState || (viewerState.is_loading && viewerState.path === path)) {
    return (
      <Stack direction="row" alignItems="center" justifyContent="center" spacing={1} sx={{ py: 4 }}>
        <CircularProgress size={20} />
        <Typography variant="body2" color="text.secondary">
          Loading file...
        </Typography>
      </Stack>
    )
  }

  if (viewerState.error && viewerState.path === path) {
    const { code } = parseErrorMessage(viewerState.error)
    const friendlyMessage = getFriendlyErrorMessage(code, path)
    return (
      <Alert severity="error" icon={<ErrorOutline fontSize="small" />}>
        <Typography variant="body2">{friendlyMessage}</Typography>
      </Alert>
    )
  }

  const content = viewerState.path === path ? viewerState.content : ''
  const binaryContent = viewerState.path === path ? viewerState.binary_content : null

  // Route to appropriate viewer based on file type
  if (fileCategory === 'image' && binaryContent) {
    return <ImageViewer path={path} />
  }

  if (fileCategory === 'video') {
    return <VideoViewer path={path} />
  }

  if (fileCategory === 'pdf' && binaryContent) {
    return <PdfViewer binaryContent={binaryContent} path={path} />
  }

  if (fileCategory === 'word' && binaryContent) {
    return <WordViewer binaryContent={binaryContent} path={path} />
  }

  if (fileCategory === 'csv' && content) {
    return <CsvViewer content={content} path={path} />
  }

  if (fileCategory === 'excel' && binaryContent) {
    return <ExcelViewer binaryContent={binaryContent} path={path} />
  }

  if (!content && !viewerState.is_loading) {
    return (
      <Box sx={{ p: 2, textAlign: 'center' }}>
        <Typography variant="body2" color="text.secondary">
          Empty file
        </Typography>
      </Box>
    )
  }

  const isMarkdown = isMarkdownFile(path)
  const language = getLanguageFromPath(path)
  const lineCount = (content || '').split('\n').length
  const useVirtualization = lineCount > VIRTUALIZATION_THRESHOLD

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
        {isMarkdown && markdownViewMode === 'preview' ? (
          <Description fontSize="small" />
        ) : (
          <Code fontSize="small" />
        )}
        <Typography variant="caption" sx={{ fontFamily: 'monospace', flex: 1 }}>
          {path.split('/').pop()}
        </Typography>
        {isMarkdown && (
          <Tooltip title={markdownViewMode === 'preview' ? 'View source' : 'View preview'}>
            <IconButton
              size="small"
              onClick={() => setMarkdownViewMode(markdownViewMode === 'preview' ? 'source' : 'preview')}
              sx={{ p: 0.25 }}
            >
              {markdownViewMode === 'preview' ? <Code sx={{ fontSize: 16 }} /> : <Visibility sx={{ fontSize: 16 }} />}
            </IconButton>
          </Tooltip>
        )}
        <Typography variant="caption" color="text.secondary">
          {isMarkdown && markdownViewMode === 'preview'
            ? 'Markdown'
            : `${lineCount.toLocaleString()} lines · ${language}`}
        </Typography>
      </Stack>
      {/* Markdown preview mode */}
      {isMarkdown && markdownViewMode === 'preview' && content ? (
        <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
          <MarkdownPreview content={content} path={path} showHeader={false} />
        </Box>
      ) : (
        /* Source code mode (with inline comments support) */
        <Box sx={{ flex: 1, minHeight: 0, overflow: useVirtualization ? 'hidden' : 'auto' }}>
          <Highlight theme={themes.vsDark} code={content || ''} language={language} prism={Prism}>
            {({ style, tokens, getTokenProps }) => {
              // Create a function to get token style
              const getTokenStyle = (token: Token): React.CSSProperties => {
                const props = getTokenProps({ token })
                return props.style || {}
              }

              const rowData: RowData = {
                tokens,
                showLineNumbers,
                onAddComment,
                commentsByLine,
                addingCommentLine,
                onLineClick: handleLineClick,
                onCommentSubmit: handleCommentSubmit,
                onCommentCancel: handleCommentCancel,
                getTokenStyle,
              }

              return (
                <Box
                  sx={{
                    height: '100%',
                    fontSize: '0.8125rem',
                    fontFamily: '"JetBrains Mono", "Fira Code", monospace',
                    lineHeight: `${LINE_HEIGHT}px`,
                    ...style,
                    background: 'transparent',
                  }}
                >
                  {useVirtualization ? <VirtualizedList rowData={rowData} /> : <SimpleCodeRenderer {...rowData} />}
                </Box>
              )
            }}
          </Highlight>
        </Box>
      )}
    </Box>
  )
}

export default SourceCodeViewer
