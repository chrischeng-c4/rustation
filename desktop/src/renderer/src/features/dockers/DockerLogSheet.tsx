import { Refresh as RefreshIcon, Close as XIcon } from '@mui/icons-material'
import {
  Button,
  Box,
  Typography,
  Drawer,
  IconButton,
  Stack,
  Divider,
  alpha
} from '@mui/material'

interface DockerLogSheetProps {
  open: boolean
  serviceName: string
  logs: string[]
  onClose: () => void
  onRefresh?: () => void
}

export function DockerLogSheet({
  open,
  serviceName,
  logs,
  onClose,
  onRefresh,
}: DockerLogSheetProps) {
  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      PaperProps={{
        sx: { width: 600, bgcolor: 'background.default' }
      }}
    >
      <Box sx={{ p: 3, height: '100%', display: 'flex', flexDirection: 'column' }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
          <Box>
            <Typography variant="h6" fontWeight={700}>{serviceName} Logs</Typography>
            <Typography variant="caption" color="text.secondary">Container output logs</Typography>
          </Box>
          <IconButton onClick={onClose}><XIcon /></IconButton>
        </Stack>
        
        <Divider sx={{ mb: 2 }} />

        <Box sx={{ 
          flex: 1, 
          overflow: 'auto', 
          my: 2, 
          p: 2, 
          bgcolor: alpha('#000', 0.3), 
          borderRadius: 2,
          border: 1,
          borderColor: 'outlineVariant'
        }}>
          <Stack spacing={0.5} sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
            {logs.length > 0 ? (
              logs.map((line, index) => (
                <Typography key={index} variant="caption" component="div" sx={{ fontFamily: 'inherit', whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {line}
                </Typography>
              ))
            ) : (
              <Typography variant="caption" color="text.disabled">No logs available</Typography>
            )}
          </Stack>
        </Box>

        <Divider sx={{ my: 2 }} />

        <Stack direction="row" justifyContent="flex-end">
          <Button
            variant="outlined"
            size="small"
            onClick={onRefresh}
            startIcon={<RefreshIcon />}
            sx={{ borderRadius: 2 }}
          >
            Refresh
          </Button>
        </Stack>
      </Box>
    </Drawer>
  )
}
