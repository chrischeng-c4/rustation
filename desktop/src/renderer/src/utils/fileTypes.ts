/**
 * File type detection utilities for media and document preview
 */

export type FileCategory =
  | 'text' // .txt, .log, code files
  | 'markdown' // .md, .mdx
  | 'image' // .png, .jpg, .gif, .svg, .webp
  | 'pdf' // .pdf
  | 'video' // .mp4, .webm, .mov, .avi
  | 'audio' // .mp3, .wav, .flac
  | 'csv' // .csv
  | 'excel' // .xlsx, .xls
  | 'word' // .docx
  | 'unknown'

/**
 * Get file category based on filename/extension
 */
export function getFileCategory(filename: string): FileCategory {
  const lowerFilename = filename.toLowerCase()
  const ext = lowerFilename.split('.').pop() || ''

  // Image files
  const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico', 'bmp']
  if (imageExts.includes(ext)) {
    return 'image'
  }

  // PDF
  if (ext === 'pdf') {
    return 'pdf'
  }

  // Video
  const videoExts = ['mp4', 'webm', 'mov', 'avi', 'mkv', 'flv']
  if (videoExts.includes(ext)) {
    return 'video'
  }

  // Audio
  const audioExts = ['mp3', 'wav', 'flac', 'ogg', 'm4a', 'aac']
  if (audioExts.includes(ext)) {
    return 'audio'
  }

  // CSV
  if (ext === 'csv') {
    return 'csv'
  }

  // Excel
  const excelExts = ['xlsx', 'xls', 'xlsm']
  if (excelExts.includes(ext)) {
    return 'excel'
  }

  // Word
  const wordExts = ['docx', 'doc']
  if (wordExts.includes(ext)) {
    return 'word'
  }

  // Markdown
  const markdownExts = ['md', 'mdx', 'markdown']
  if (markdownExts.includes(ext)) {
    return 'markdown'
  }

  // Text files (including code)
  const textExts = [
    'txt',
    'log',
    'json',
    'xml',
    'yaml',
    'yml',
    'toml',
    'ini',
    'conf',
    'config',
    'env',
    // Programming languages
    'ts',
    'tsx',
    'js',
    'jsx',
    'py',
    'rs',
    'go',
    'java',
    'c',
    'cpp',
    'h',
    'hpp',
    'cs',
    'php',
    'rb',
    'swift',
    'kt',
    'scala',
    'sh',
    'bash',
    'zsh',
    'fish',
    // Web
    'html',
    'htm',
    'css',
    'scss',
    'sass',
    'less',
    'vue',
    'svelte',
    // Other
    'sql',
    'graphql',
    'proto',
    'dockerfile',
    'makefile',
    'cmake',
  ]
  if (textExts.includes(ext)) {
    return 'text'
  }

  // Special filenames
  const specialTextFiles = [
    'dockerfile',
    'makefile',
    'cmakelists.txt',
    'readme',
    'changelog',
    'license',
    'authors',
    'contributing',
    'dockerfile',
    '.gitignore',
    '.dockerignore',
    '.editorconfig',
    '.eslintrc',
    '.prettierrc',
  ]
  if (specialTextFiles.includes(lowerFilename)) {
    return 'text'
  }

  return 'unknown'
}

/**
 * Check if a file category requires binary file reading
 */
export function isBinaryFile(category: FileCategory): boolean {
  return ['image', 'pdf', 'video', 'audio', 'excel', 'word'].includes(category)
}

/**
 * Check if file size is considered large (>5MB)
 */
export function isLargeFile(sizeBytes: number): boolean {
  const LARGE_FILE_THRESHOLD = 5 * 1024 * 1024 // 5MB
  return sizeBytes > LARGE_FILE_THRESHOLD
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}

/**
 * Get MIME type for a file category
 */
export function getMimeType(category: FileCategory, filename: string): string {
  const ext = filename.toLowerCase().split('.').pop() || ''

  switch (category) {
    case 'image':
      if (ext === 'svg') return 'image/svg+xml'
      if (ext === 'png') return 'image/png'
      if (ext === 'jpg' || ext === 'jpeg') return 'image/jpeg'
      if (ext === 'gif') return 'image/gif'
      if (ext === 'webp') return 'image/webp'
      return 'image/*'

    case 'pdf':
      return 'application/pdf'

    case 'video':
      if (ext === 'mp4') return 'video/mp4'
      if (ext === 'webm') return 'video/webm'
      if (ext === 'mov') return 'video/quicktime'
      return 'video/*'

    case 'audio':
      if (ext === 'mp3') return 'audio/mpeg'
      if (ext === 'wav') return 'audio/wav'
      if (ext === 'flac') return 'audio/flac'
      if (ext === 'ogg') return 'audio/ogg'
      return 'audio/*'

    case 'csv':
      return 'text/csv'

    case 'excel':
      if (ext === 'xlsx') return 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      if (ext === 'xls') return 'application/vnd.ms-excel'
      return 'application/vnd.ms-excel'

    case 'word':
      if (ext === 'docx')
        return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      if (ext === 'doc') return 'application/msword'
      return 'application/msword'

    case 'markdown':
      return 'text/markdown'

    case 'text':
      return 'text/plain'

    default:
      return 'application/octet-stream'
  }
}
