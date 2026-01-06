import { useState, useCallback, useRef, useEffect } from 'react'
import { Box, Button, IconButton, Paper, Stack, TextField, Typography } from '@mui/material'
import { Autorenew, ChatBubbleOutline, DeleteOutline, Person, Send, SmartToy } from '@mui/icons-material'
import { PageHeader } from '@/components/shared/PageHeader'
import { LoadingState } from '@/components/shared/LoadingState'
import { EmptyState } from '@/components/shared/EmptyState'
import { ErrorBanner } from '@/components/shared/ErrorBanner'
import { useChatState } from '@/hooks/useAppState'
import type { ChatMessage } from '@/types/state'

/**
 * Chat Page for Claude AI interaction.
 * Provides a chat interface with project context via MCP.
 */
export function ChatPage() {
  const { chat, projectName, dispatch, isLoading } = useChatState()
  const [inputValue, setInputValue] = useState('')
  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [chat?.messages])

  const handleSend = useCallback(async () => {
    if (!inputValue.trim() || chat?.is_typing) return

    const text = inputValue.trim()
    setInputValue('')

    // Generate a unique ID for the user message
    const messageId = `user-${Date.now()}`
    const timestamp = new Date().toISOString()

    // Add user message immediately
    await dispatch({
      type: 'AddChatMessage',
      payload: {
        message: {
          id: messageId,
          role: 'user',
          content: text,
          timestamp,
        },
      },
    })

    // Trigger sending to Claude (this will set is_typing and handle response)
    await dispatch({
      type: 'SendChatMessage',
      payload: { text },
    })
  }, [inputValue, chat?.is_typing, dispatch])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        handleSend()
      }
    },
    [handleSend]
  )

  const handleClear = useCallback(async () => {
    await dispatch({ type: 'ClearChat' })
  }, [dispatch])

  const handleClearError = useCallback(async () => {
    await dispatch({ type: 'ClearChatError' })
  }, [dispatch])

  // Loading state
  if (isLoading) {
    return <LoadingState message="Connecting to AI assistant..." />
  }

  // No project open
  if (!chat) {
    return (
      <EmptyState
        icon={ChatBubbleOutline}
        title="No Project Open"
        description="Open a project to start chatting with Claude."
      />
    )
  }

  const messages = chat.messages ?? []
  const isTyping = chat.is_typing
  const error = chat.error

  return (
    <Stack sx={{ height: '100%' }}>
      {/* Header */}
      <PageHeader
        title="Chat"
        description={`Chat with Claude about ${projectName}`}
        icon={<ChatBubbleOutline fontSize="small" />}
      >
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          disabled={messages.length === 0}
          startIcon={<DeleteOutline fontSize="small" />}
        >
          Clear Chat
        </Button>
      </PageHeader>

      {/* Messages Area */}
      <Box ref={scrollRef} sx={{ flex: 1, overflow: 'auto', p: 2, pt: 0 }}>
        {messages.length === 0 ? (
          <EmptyState
            icon={SmartToy}
            title="Start a conversation"
            description="Ask questions about your project, generate code, or get help with development tasks."
            sx={{ py: 6 }}
          />
        ) : (
          <Stack spacing={2}>
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {isTyping && (
              <Stack direction="row" alignItems="center" spacing={1} sx={{ pl: 1 }}>
                <Autorenew fontSize="small" sx={{ color: 'text.secondary', animation: 'spin 1s linear infinite' }} />
                <Typography variant="body2" color="text.secondary">
                  Claude is thinking...
                </Typography>
              </Stack>
            )}
          </Stack>
        )}
      </Box>

      {/* Error Display */}
      {error && (
        <Box sx={{ px: 2, mb: 2 }}>
          <ErrorBanner error={error} />
          <Button variant="text" size="small" onClick={handleClearError} sx={{ mt: 1 }}>
            Dismiss Error
          </Button>
        </Box>
      )}

      {/* Input Area */}
      <Box sx={{ borderTop: 1, borderColor: 'divider', p: 2 }}>
        <Stack direction="row" spacing={2}>
          <TextField
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ask Claude about your project..."
            multiline
            minRows={3}
            fullWidth
            disabled={isTyping}
          />
          <IconButton
            color="primary"
            onClick={handleSend}
            disabled={!inputValue.trim() || isTyping}
            sx={{ alignSelf: 'stretch' }}
          >
            {isTyping ? (
              <Autorenew fontSize="small" sx={{ animation: 'spin 1s linear infinite' }} />
            ) : (
              <Send fontSize="small" />
            )}
          </IconButton>
        </Stack>
        <Typography variant="caption" color="text.secondary" align="center" sx={{ mt: 1, display: 'block' }}>
          Press Enter to send, Shift+Enter for new line
        </Typography>
      </Box>
    </Stack>
  )
}

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === 'user'
  const isSystem = message.role === 'system'

  return (
    <Stack direction={isUser ? 'row-reverse' : 'row'} spacing={1.5} alignItems="flex-start">
      {/* Avatar */}
      <Box
        sx={{
          height: 32,
          width: 32,
          borderRadius: '50%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: isUser ? 'primary.main' : isSystem ? 'action.hover' : 'secondary.main',
          color: isUser ? 'primary.contrastText' : isSystem ? 'text.secondary' : 'secondary.contrastText',
          flexShrink: 0,
        }}
      >
        {isUser ? <Person fontSize="small" /> : <SmartToy fontSize="small" />}
      </Box>

      {/* Message Content */}
      <Paper
        variant="outlined"
        sx={{
          maxWidth: '80%',
          px: 2,
          py: 1,
          bgcolor: isUser ? 'primary.main' : isSystem ? 'action.hover' : 'background.paper',
          color: isUser ? 'primary.contrastText' : 'text.primary',
        }}
      >
        <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
          {message.content}
        </Typography>
        {message.is_streaming && (
          <Box component="span" sx={{ display: 'inline-block', width: 8, height: 16, ml: 0.5, bgcolor: 'currentColor', animation: 'pulse 1s ease-in-out infinite' }} />
        )}
      </Paper>
    </Stack>
  )
}
