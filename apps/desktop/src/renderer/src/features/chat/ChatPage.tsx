import { useState, useCallback, useRef, useEffect } from 'react'
import {
  MessageSquare,
  Send,
  RefreshCw,
  Trash2,
  User,
  Bot,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
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
        icon={MessageSquare}
        title="No Project Open"
        description="Open a project to start chatting with Claude."
      />
    )
  }

  const messages = chat.messages ?? []
  const isTyping = chat.is_typing
  const error = chat.error

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <PageHeader
        title="Chat"
        description={`Chat with Claude about ${projectName}`}
        icon={<MessageSquare className="h-5 w-5" />}
      >
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          disabled={messages.length === 0}
          className="h-8 gap-1.5"
        >
          <Trash2 className="h-3.5 w-3.5" />
          Clear Chat
        </Button>
      </PageHeader>

      {/* Messages Area */}
      <ScrollArea className="flex-1 p-4 pt-0" ref={scrollRef}>
        {messages.length === 0 ? (
          <EmptyState
            icon={Bot}
            title="Start a conversation"
            description="Ask questions about your project, generate code, or get help with development tasks."
            className="py-12"
          />
        ) : (
          <div className="space-y-4">
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {isTyping && (
              <div className="flex items-center gap-2 text-muted-foreground text-sm pl-2">
                <RefreshCw className="h-3 w-3 animate-spin" />
                <span>Claude is thinking...</span>
              </div>
            )}
          </div>
        )}
      </ScrollArea>

      {/* Error Display */}
      {error && (
        <div className="px-4 mb-2">
          <ErrorBanner error={error} />
          <Button variant="ghost" size="sm" onClick={handleClearError} className="mt-1 h-7 text-xs">
            Dismiss Error
          </Button>
        </div>
      )}

      {/* Input Area */}
      <div className="border-t p-4">
        <div className="flex gap-2">
          <Textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ask Claude about your project..."
            className="min-h-[80px] resize-none focus-visible:ring-primary/20"
            disabled={isTyping}
          />
          <Button
            onClick={handleSend}
            disabled={!inputValue.trim() || isTyping}
            className="shrink-0 h-auto self-stretch px-4"
          >
            {isTyping ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <Send className="h-4 w-4" />
            )}
          </Button>
        </div>
        <p className="mt-2 text-[10px] text-muted-foreground text-center">
          Press <kbd className="pointer-events-none inline-flex h-4 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">Enter</kbd> to send, <kbd className="pointer-events-none inline-flex h-4 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">Shift+Enter</kbd> for new line
        </p>
      </div>
    </div>
  )
}

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === 'user'
  const isSystem = message.role === 'system'

  return (
    <div
      className={`flex items-start gap-3 ${isUser ? 'flex-row-reverse' : ''}`}
    >
      {/* Avatar */}
      <div
        className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${
          isUser
            ? 'bg-primary text-primary-foreground'
            : isSystem
              ? 'bg-muted text-muted-foreground'
              : 'bg-violet-500 text-white'
        }`}
      >
        {isUser ? (
          <User className="h-4 w-4" />
        ) : (
          <Bot className="h-4 w-4" />
        )}
      </div>

      {/* Message Content */}
      <Card
        className={`max-w-[80%] px-4 py-2 ${
          isUser
            ? 'bg-primary text-primary-foreground'
            : isSystem
              ? 'bg-muted'
              : 'bg-card'
        }`}
      >
        <div className="whitespace-pre-wrap break-words">{message.content}</div>
        {message.is_streaming && (
          <span className="inline-block w-2 h-4 ml-1 bg-current animate-pulse" />
        )}
      </Card>
    </div>
  )
}
