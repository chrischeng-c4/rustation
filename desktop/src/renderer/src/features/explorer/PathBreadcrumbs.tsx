import {
  ChevronRight as ChevronRightIcon,
  Home as HomeIcon,
  Folder as FolderIcon
} from '@mui/icons-material'
import { Button, Breadcrumbs, Typography } from '@mui/material'

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
    <Breadcrumbs
      separator={<ChevronRightIcon sx={{ fontSize: 14, color: 'text.disabled' }} />}
      aria-label="breadcrumb"
      sx={{
        '& .MuiBreadcrumbs-ol': { flexWrap: 'nowrap' },
        '& .MuiBreadcrumbs-li': { minWidth: 0 }
      }}
    >
      <Button
        variant="text"
        size="small"
        onClick={() => onNavigate(rootPath)}
        startIcon={<HomeIcon sx={{ fontSize: 16 }} />}
        sx={{
          minHeight: 28,
          px: 1,
          textTransform: 'none',
          color: 'primary.main',
          fontWeight: 700,
          fontSize: '0.75rem',
          borderRadius: 1
        }}
      >
        Project
      </Button>

      {segments.map((segment, idx) => {
        const isLast = idx === segments.length - 1
        const fullPath = rootPath + '/' + segments.slice(0, idx + 1).join('/')

        return (
          <Button
            key={fullPath}
            variant="text"
            size="small"
            onClick={() => onNavigate(fullPath)}
            startIcon={!isLast ? <FolderIcon sx={{ fontSize: 14, opacity: 0.7 }} /> : null}
            sx={{
              minHeight: 28,
              px: 1,
              textTransform: 'none',
              color: isLast ? 'text.primary' : 'text.secondary',
              fontWeight: isLast ? 700 : 500,
              fontSize: '0.75rem',
              minWidth: 0,
              maxWidth: 150,
              borderRadius: 1,
              '& .MuiButton-startIcon': { mr: 0.5 }
            }}
          >
            <Typography variant="caption" fontWeight="inherit" noWrap>
              {segment}
            </Typography>
          </Button>
        )
      })}
    </Breadcrumbs>
  )
}
