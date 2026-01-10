/**
 * File icon mapping utilities
 * Maps filenames and extensions to material-icon-theme icon names
 */

export type IconName =
  // Languages
  | 'typescript'
  | 'javascript'
  | 'rust'
  | 'python'
  | 'go'
  | 'java'
  | 'kotlin'
  | 'swift'
  | 'c'
  | 'cpp'
  | 'csharp'
  // Web
  | 'html'
  | 'css'
  | 'scss'
  | 'sass'
  | 'vue'
  | 'react'
  | 'angular'
  | 'svelte'
  // Data formats
  | 'json'
  | 'yaml'
  | 'toml'
  | 'xml'
  // DevOps
  | 'docker'
  | 'kubernetes'
  | 'terraform'
  | 'git'
  | 'github'
  // Build/Config
  | 'npm'
  | 'cargo'
  | 'makefile'
  | 'env'
  | 'settings'
  // Documentation
  | 'markdown'
  | 'pdf'
  | 'text'
  // Media
  | 'image'
  | 'video'
  | 'audio'
  // Archive
  | 'zip'
  // Folders
  | 'folder'
  | 'folder-open'
  | 'folder-git'
  | 'folder-git-open'
  | 'folder-node'
  | 'folder-node-open'
  | 'folder-src'
  | 'folder-src-open'
  | 'folder-test'
  | 'folder-test-open'
  | 'folder-github'
  | 'folder-github-open'
  | 'folder-docker'
  | 'folder-docker-open'
  // Fallback
  | 'file'

/**
 * Get icon name for a file based on its filename
 */
export function getFileIconName(filename: string): IconName {
  const lowerFilename = filename.toLowerCase()
  const ext = lowerFilename.split('.').pop() || ''

  // Handle special filenames without extensions
  const filenameMap: Record<string, IconName> = {
    // Docker
    dockerfile: 'docker',
    'docker-compose.yml': 'docker',
    'docker-compose.yaml': 'docker',
    '.dockerignore': 'docker',
    // Kubernetes
    'kustomization.yaml': 'kubernetes',
    'kustomization.yml': 'kubernetes',
    // Git
    '.gitignore': 'git',
    '.gitattributes': 'git',
    '.gitmodules': 'git',
    // Env files
    '.env': 'env',
    '.env.local': 'env',
    '.env.development': 'env',
    '.env.production': 'env',
    '.env.test': 'env',
    // Package files
    'package.json': 'npm',
    'package-lock.json': 'npm',
    'cargo.toml': 'cargo',
    'cargo.lock': 'cargo',
    // Build files
    makefile: 'makefile',
    justfile: 'makefile',
    // Markdown
    'readme.md': 'markdown',
    'changelog.md': 'markdown',
  }

  // Check exact filename match first
  if (filenameMap[lowerFilename]) {
    return filenameMap[lowerFilename]
  }

  // Check if it's a Kubernetes manifest (*.k8s.yaml pattern)
  if (lowerFilename.includes('.k8s.yaml') || lowerFilename.includes('.k8s.yml')) {
    return 'kubernetes'
  }

  // Check extensions
  const extensionMap: Record<string, IconName> = {
    // TypeScript/JavaScript
    ts: 'typescript',
    tsx: 'react', // TypeScript React uses React icon
    js: 'javascript',
    jsx: 'react',
    mjs: 'javascript',
    cjs: 'javascript',
    // Rust
    rs: 'rust',
    // Python
    py: 'python',
    pyw: 'python',
    pyx: 'python',
    pyi: 'python',
    // Go
    go: 'go',
    mod: 'go',
    // Java/Kotlin
    java: 'java',
    kt: 'kotlin',
    kts: 'kotlin',
    // Swift
    swift: 'swift',
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
    // Web
    html: 'html',
    htm: 'html',
    css: 'css',
    scss: 'scss',
    sass: 'sass',
    vue: 'vue',
    svelte: 'svelte',
    angular: 'angular',
    // Data formats
    json: 'json',
    json5: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    toml: 'toml',
    xml: 'xml',
    // DevOps
    tf: 'terraform',
    tfvars: 'terraform',
    // Documentation
    md: 'markdown',
    mdx: 'markdown',
    pdf: 'pdf',
    txt: 'text',
    // Media
    png: 'image',
    jpg: 'image',
    jpeg: 'image',
    gif: 'image',
    svg: 'image',
    webp: 'image',
    ico: 'image',
    mp4: 'video',
    webm: 'video',
    mov: 'video',
    avi: 'video',
    mp3: 'audio',
    wav: 'audio',
    flac: 'audio',
    ogg: 'audio',
    // Archive
    zip: 'zip',
    tar: 'zip',
    gz: 'zip',
    rar: 'zip',
    '7z': 'zip',
  }

  return extensionMap[ext] || 'file'
}

/**
 * Get icon name for a folder based on its name
 */
export function getFolderIconName(folderName: string, isOpen: boolean): IconName {
  const lowerName = folderName.toLowerCase()

  // Special folder mappings
  const folderMap: Record<string, string> = {
    '.git': 'folder-git',
    '.github': 'folder-github',
    node_modules: 'folder-node',
    src: 'folder-src',
    test: 'folder-test',
    tests: 'folder-test',
    __tests__: 'folder-test',
    docker: 'folder-docker',
  }

  const baseName = folderMap[lowerName] || 'folder'
  return (isOpen ? `${baseName}-open` : baseName) as IconName
}
