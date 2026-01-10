import { memo, useMemo } from 'react'
import { getFileIconName, getFolderIconName, type IconName } from '@/utils/fileIcons'

// Import SVG icons as React components
import FileIconSvg from 'material-icon-theme/icons/file.svg?react'
import FolderIconSvg from 'material-icon-theme/icons/folder.svg?react'
import FolderOpenIconSvg from 'material-icon-theme/icons/folder-open.svg?react'

// Languages
import TypeScriptIconSvg from 'material-icon-theme/icons/typescript.svg?react'
import JavaScriptIconSvg from 'material-icon-theme/icons/javascript.svg?react'
import ReactIconSvg from 'material-icon-theme/icons/react.svg?react'
import RustIconSvg from 'material-icon-theme/icons/rust.svg?react'
import PythonIconSvg from 'material-icon-theme/icons/python.svg?react'
import GoIconSvg from 'material-icon-theme/icons/go.svg?react'
import JavaIconSvg from 'material-icon-theme/icons/java.svg?react'
import KotlinIconSvg from 'material-icon-theme/icons/kotlin.svg?react'
import SwiftIconSvg from 'material-icon-theme/icons/swift.svg?react'
import CIconSvg from 'material-icon-theme/icons/c.svg?react'
import CppIconSvg from 'material-icon-theme/icons/cpp.svg?react'
import CsharpIconSvg from 'material-icon-theme/icons/csharp.svg?react'

// Web
import HtmlIconSvg from 'material-icon-theme/icons/html.svg?react'
import CssIconSvg from 'material-icon-theme/icons/css.svg?react'
import SassIconSvg from 'material-icon-theme/icons/sass.svg?react' // Use Sass icon for both SCSS and Sass
import VueIconSvg from 'material-icon-theme/icons/vue.svg?react'
import AngularIconSvg from 'material-icon-theme/icons/angular.svg?react'
import SvelteIconSvg from 'material-icon-theme/icons/svelte.svg?react'

// Data formats
import JsonIconSvg from 'material-icon-theme/icons/json.svg?react'
import YamlIconSvg from 'material-icon-theme/icons/yaml.svg?react'
import TomlIconSvg from 'material-icon-theme/icons/toml.svg?react'
import XmlIconSvg from 'material-icon-theme/icons/xml.svg?react'

// DevOps
import DockerIconSvg from 'material-icon-theme/icons/docker.svg?react'
import KubernetesIconSvg from 'material-icon-theme/icons/kubernetes.svg?react'
import TerraformIconSvg from 'material-icon-theme/icons/terraform.svg?react'
import GitIconSvg from 'material-icon-theme/icons/git.svg?react'

// Build/Config
import NpmIconSvg from 'material-icon-theme/icons/npm.svg?react'
import CargoIconSvg from 'material-icon-theme/icons/rust.svg?react' // Use Rust icon for Cargo
import MakefileIconSvg from 'material-icon-theme/icons/makefile.svg?react'
import EnvIconSvg from 'material-icon-theme/icons/tune.svg?react'
import SettingsIconSvg from 'material-icon-theme/icons/settings.svg?react'

// Documentation
import MarkdownIconSvg from 'material-icon-theme/icons/markdown.svg?react'
import PdfIconSvg from 'material-icon-theme/icons/pdf.svg?react'
import TextIconSvg from 'material-icon-theme/icons/document.svg?react'

// Media
import ImageIconSvg from 'material-icon-theme/icons/image.svg?react'
import VideoIconSvg from 'material-icon-theme/icons/video.svg?react'
import AudioIconSvg from 'material-icon-theme/icons/audio.svg?react'

// Archive
import ZipIconSvg from 'material-icon-theme/icons/zip.svg?react'

// Special folders
import FolderGitIconSvg from 'material-icon-theme/icons/folder-git.svg?react'
import FolderGitOpenIconSvg from 'material-icon-theme/icons/folder-git-open.svg?react'
import FolderNodeIconSvg from 'material-icon-theme/icons/folder-node.svg?react'
import FolderNodeOpenIconSvg from 'material-icon-theme/icons/folder-node-open.svg?react'
import FolderSrcIconSvg from 'material-icon-theme/icons/folder-src.svg?react'
import FolderSrcOpenIconSvg from 'material-icon-theme/icons/folder-src-open.svg?react'
import FolderTestIconSvg from 'material-icon-theme/icons/folder-test.svg?react'
import FolderTestOpenIconSvg from 'material-icon-theme/icons/folder-test-open.svg?react'
import FolderGithubIconSvg from 'material-icon-theme/icons/folder-github.svg?react'
import FolderGithubOpenIconSvg from 'material-icon-theme/icons/folder-github-open.svg?react'
import FolderDockerIconSvg from 'material-icon-theme/icons/folder-docker.svg?react'
import FolderDockerOpenIconSvg from 'material-icon-theme/icons/folder-docker-open.svg?react'

// Icon map
const iconMap: Record<IconName, React.FC<React.SVGProps<SVGSVGElement>>> = {
  // Languages
  typescript: TypeScriptIconSvg,
  javascript: JavaScriptIconSvg,
  react: ReactIconSvg,
  rust: RustIconSvg,
  python: PythonIconSvg,
  go: GoIconSvg,
  java: JavaIconSvg,
  kotlin: KotlinIconSvg,
  swift: SwiftIconSvg,
  c: CIconSvg,
  cpp: CppIconSvg,
  csharp: CsharpIconSvg,
  // Web
  html: HtmlIconSvg,
  css: CssIconSvg,
  scss: SassIconSvg, // Use Sass icon for SCSS
  sass: SassIconSvg,
  vue: VueIconSvg,
  angular: AngularIconSvg,
  svelte: SvelteIconSvg,
  // Data formats
  json: JsonIconSvg,
  yaml: YamlIconSvg,
  toml: TomlIconSvg,
  xml: XmlIconSvg,
  // DevOps
  docker: DockerIconSvg,
  kubernetes: KubernetesIconSvg,
  terraform: TerraformIconSvg,
  git: GitIconSvg,
  github: GitIconSvg, // Use Git icon for GitHub files
  // Build/Config
  npm: NpmIconSvg,
  cargo: CargoIconSvg,
  makefile: MakefileIconSvg,
  env: EnvIconSvg,
  settings: SettingsIconSvg,
  // Documentation
  markdown: MarkdownIconSvg,
  pdf: PdfIconSvg,
  text: TextIconSvg,
  // Media
  image: ImageIconSvg,
  video: VideoIconSvg,
  audio: AudioIconSvg,
  // Archive
  zip: ZipIconSvg,
  // Folders
  folder: FolderIconSvg,
  'folder-open': FolderOpenIconSvg,
  'folder-git': FolderGitIconSvg,
  'folder-git-open': FolderGitOpenIconSvg,
  'folder-node': FolderNodeIconSvg,
  'folder-node-open': FolderNodeOpenIconSvg,
  'folder-src': FolderSrcIconSvg,
  'folder-src-open': FolderSrcOpenIconSvg,
  'folder-test': FolderTestIconSvg,
  'folder-test-open': FolderTestOpenIconSvg,
  'folder-github': FolderGithubIconSvg,
  'folder-github-open': FolderGithubOpenIconSvg,
  'folder-docker': FolderDockerIconSvg,
  'folder-docker-open': FolderDockerOpenIconSvg,
  // Fallback
  file: FileIconSvg,
}

export interface FileIconProps {
  filename: string
  kind: 'file' | 'directory' | 'symlink'
  isOpen?: boolean
  size?: number
  className?: string
  style?: React.CSSProperties
}

/**
 * FileIcon component - displays file-type-specific icons from material-icon-theme
 * Memoized for performance in large file lists
 */
export const FileIcon = memo(function FileIcon({
  filename,
  kind,
  isOpen = false,
  size = 16,
  className,
  style,
}: FileIconProps) {
  const iconName = useMemo(() => {
    if (kind === 'directory') {
      return getFolderIconName(filename, isOpen)
    }
    return getFileIconName(filename)
  }, [filename, kind, isOpen])

  const IconComponent = iconMap[iconName] || iconMap.file

  return (
    <IconComponent
      width={size}
      height={size}
      className={className}
      style={{
        flexShrink: 0,
        display: 'inline-block',
        verticalAlign: 'middle',
        ...style,
      }}
    />
  )
})
