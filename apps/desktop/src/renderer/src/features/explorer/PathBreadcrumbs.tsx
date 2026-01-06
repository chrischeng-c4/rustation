import { ChevronRight, Home } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface PathBreadcrumbsProps {
  currentPath: string
  rootPath: string
  onNavigate: (path: string) => void
}

export function PathBreadcrumbs({ currentPath, rootPath, onNavigate }: PathBreadcrumbsProps) {
  // Calculate relative path segments
  const relativePath = currentPath.startsWith(rootPath) 
    ? currentPath.slice(rootPath.length) 
    : ''
  
  const segments = relativePath.split('/').filter(Boolean)
  
  return (
    <div className="flex items-center text-sm overflow-x-auto whitespace-nowrap no-scrollbar">
      <Button 
        variant="ghost" 
        size="sm" 
        className="h-7 px-2 flex items-center gap-1"
        onClick={() => onNavigate(rootPath)}
      >
        <Home className="h-3.5 w-3.5" />
        <span className="max-w-[100px] truncate">Project</span>
      </Button>

      {segments.map((segment, idx) => {
        const fullPath = rootPath + '/' + segments.slice(0, idx + 1).join('/')
        return (
          <div key={fullPath} className="flex items-center">
            <ChevronRight className="h-3.5 w-3.5 text-muted-foreground mx-0.5" />
            <Button 
              variant="ghost" 
              size="sm" 
              className="h-7 px-2"
              onClick={() => onNavigate(fullPath)}
            >
              {segment}
            </Button>
          </div>
        )
      })}
    </div>
  )
}
