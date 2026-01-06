import { useState, useCallback } from 'react'
import { 
  InfoOutlined as InfoIcon, 
  Code as FileCode, 
  ChatBubbleOutline as MessageSquare, 
  Send, 
  AccountCircle as UserIcon 
} from '@mui/icons-material'
import { 
  Box, 
  Tabs, 
  Tab, 
  Typography, 
  Divider, 
  IconButton, 
  TextField, 
  Button,
  Paper,
  Avatar,
  Stack
} from '@mui/material'
import { useActiveWorktree } from '@/hooks/useAppState'
import ReactMarkdown from 'react-markdown'

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function CustomTabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`explorer-tabpanel-${index}`}
      aria-labelledby={`explorer-tab-${index}`}
      style={{ flex: 1, minHeight: 0, display: value === index ? 'flex' : 'none', flexDirection: 'column' }}
      {...other}
    >
      {value === index && children}
    </div>
  )
}

export function DetailPanel() {
  const { worktree, dispatch } = useActiveWorktree()
  const explorer = worktree?.explorer
  const selectedPath = explorer?.selected_path
  const selectedEntry = explorer?.entries.find((e) => e.path === selectedPath)
  const comments = explorer?.selected_comments ?? []
  
  const [tabValue, setTabValue] = useState(2) // Default to Comments
  const [newComment, setNewComment] = useState('')

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue)
  }

  const handleAddComment = useCallback(async () => {
    if (!selectedPath || !newComment.trim()) return
    
    await dispatch({
      type: 'AddFileComment',
      payload: { path: selectedPath, content: newComment.trim() }
    })
    setNewComment('')
  }, [selectedPath, newComment, dispatch])

  if (!selectedPath || !selectedEntry) {
    return (
      <Box sx={{ 
        display: 'flex', 
        height: '100%', 
        alignItems: 'center', 
        justifyContent: 'center', 
        p: 4, 
        textAlign: 'center', 
        color: 'text.secondary' 
      }}>
        <Box>
          <InfoIcon sx={{ fontSize: 48, mb: 2, opacity: 0.1 }} />
          <Typography variant="body2">Select a file to view details</Typography>
        </Box>
      </Box>
    )
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', overflow: 'hidden', borderLeft: 1, borderColor: 'divider' }}>
      <Box sx={{ borderBottom: 1, borderColor: 'divider', bgcolor: 'background.paper', opacity: 0.5 }}>
        <Tabs 
          value={tabValue} 
          onChange={handleTabChange} 
          variant="fullWidth" 
          size="small"
          sx={{ minHeight: 40 }}
        >
          <Tab icon={<InfoIcon sx={{ fontSize: 18 }} />} iconPosition="start" label="Info" sx={{ minHeight: 40, fontSize: '0.7rem' }} />
          <Tab icon={<FileCode sx={{ fontSize: 18 }} />} iconPosition="start" label="Preview" sx={{ minHeight: 40, fontSize: '0.7rem' }} />
          <Tab icon={<MessageSquare sx={{ fontSize: 18 }} />} iconPosition="start" label="Comments" sx={{ minHeight: 40, fontSize: '0.7rem' }} />
        </Tabs>
      </Box>

      {/* Info Tab */}
      <CustomTabPanel value={tabValue} index={0}>
        <Box sx={{ p: 2, overflow: 'auto', flex: 1 }}>
          <Stack spacing={3}>
            <Box>
              <Typography variant="caption" sx={{ fontWeight: 600, color: 'primary.main', textTransform: 'uppercase', letterSpacing: 1 }}>
                Metadata
              </Typography>
              <Stack spacing={1.5} sx={{ mt: 1.5 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" color="text.secondary">Name</Typography>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>{selectedEntry.name}</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" color="text.secondary">Kind</Typography>
                  <Typography variant="body2" sx={{ textTransform: 'capitalize' }}>{selectedEntry.kind}</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" color="text.secondary">Size</Typography>
                  <Typography variant="body2">{selectedEntry.kind === 'file' ? (selectedEntry.size / 1024).toFixed(1) + ' KB' : '--'}</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" color="text.secondary">Permissions</Typography>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace', bgcolor: 'action.hover', px: 0.5, borderRadius: 0.5 }}>{selectedEntry.permissions}</Typography>
                </Box>
              </Stack>
            </Box>
            
            <Divider />
            
            <Box>
              <Typography variant="caption" sx={{ fontWeight: 600, color: 'text.secondary', textTransform: 'uppercase', letterSpacing: 1 }}>
                Full Path
              </Typography>
              <Paper variant="outlined" sx={{ mt: 1, p: 1.5, bgcolor: 'action.hover', borderStyle: 'dashed' }}>
                <Typography variant="caption" sx={{ fontFamily: 'monospace', wordBreak: 'break-all', display: 'block', lineHeight: 1.6 }}>
                  {selectedEntry.path}
                </Typography>
              </Paper>
            </Box>
          </Stack>
        </Box>
      </CustomTabPanel>

      {/* Preview Tab */}
      <CustomTabPanel value={tabValue} index={1}>
        <Box sx={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', p: 2 }}>
          <Paper variant="outlined" sx={{ 
            width: '100%', 
            height: '100%', 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center',
            bgcolor: 'transparent',
            borderStyle: 'dashed'
          }}>
            <Box sx={{ textAlign: 'center', color: 'text.secondary', opacity: 0.5 }}>
              <FileCode sx={{ fontSize: 40, mb: 1 }} />
              <Typography variant="caption" display="block">Preview will use SourceCodeViewer</Typography>
            </Box>
          </Paper>
        </Box>
      </CustomTabPanel>

      {/* Comments Tab */}
      <CustomTabPanel value={tabValue} index={2}>
        <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
          <Stack spacing={2.5}>
            {comments.map((comment) => (
              <Box key={comment.id} sx={{ display: 'flex', gap: 1.5 }}>
                <Avatar sx={{ width: 28, height: 28, bgcolor: 'primary.dark', color: 'primary.main', fontSize: 14 }}>
                  <UserIcon sx={{ fontSize: 18 }} />
                </Avatar>
                <Box sx={{ flex: 1, minWidth: 0 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 0.5 }}>
                    <Typography variant="caption" sx={{ fontWeight: 600 }}>{comment.author}</Typography>
                    <Typography variant="caption" sx={{ fontSize: 9, color: 'text.secondary' }}>
                      {new Date(comment.created_at).toLocaleTimeString()}
                    </Typography>
                  </Box>
                  <Box className="prose prose-sm dark:prose-invert" sx={{ 
                    '& p': { m: 0, fontSize: '0.8rem', lineHeight: 1.5, color: 'text.primary' } 
                  }}>
                    <ReactMarkdown>{comment.content}</ReactMarkdown>
                  </Box>
                </Box>
              </Box>
            ))}
            {comments.length === 0 && (
              <Box sx={{ textAlign: 'center', py: 6, color: 'text.secondary', opacity: 0.3 }}>
                <MessageSquare sx={{ fontSize: 40, mb: 1 }} />
                <Typography variant="caption" display="block">No comments yet</Typography>
              </Box>
            )}
          </Stack>
        </Box>

        <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider', bgcolor: 'background.paper' }}>
          <TextField
            multiline
            rows={3}
            fullWidth
            placeholder="Add a note for the AI..."
            value={newComment}
            onChange={(e) => setNewComment(e.target.value)}
            variant="outlined"
            size="small"
            sx={{ 
              '& .MuiOutlinedInput-root': { 
                fontSize: '0.8rem',
                borderRadius: 2,
                bgcolor: 'background.default'
              }
            }}
          />
          <Button 
            fullWidth 
            variant="contained" 
            size="small"
            disabled={!newComment.trim()}
            onClick={handleAddComment}
            startIcon={<Send sx={{ fontSize: 14 }} />}
            sx={{ mt: 1, height: 32, borderRadius: 2 }}
          >
            Post Comment
          </Button>
        </Box>
      </CustomTabPanel>
    </Box>
  )
}
