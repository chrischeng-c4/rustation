import { useState, useCallback, useRef, useEffect } from 'react'
import {
  MessageSquare,
  Send,
  RefreshCw,
  Trash2,
  AlertCircle,
  User,
  Bot,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
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
    return (
      <div className="flex h-full items-center justify-center">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // No project open
  if (!chat) {
    return (
      <div className="flex h-full flex-col items-center justify-center">
        <MessageSquare className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">No Project Open</h2>
        <p className="mt-2 text-muted-foreground">
          Open a project to start chatting with Claude.
        </p>
      </div>
    )
  }

  const messages = chat.messages ?? []
  const isTyping = chat.is_typing
  const error = chat.error

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b px-4 py-3">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <MessageSquare className="h-5 w-5" />
            Chat
          </h2>
          <p className="text-sm text-muted-foreground">
            Chat with Claude about {projectName}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          disabled={messages.length === 0}
        >
          <Trash2 className="mr-2 h-4 w-4" />
          Clear
        </Button>
      </div>

      {/* Messages Area */}
      <ScrollArea className="flex-1 p-4" ref={scrollRef}>
        {messages.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center py-12">
            <Bot className="h-16 w-16 text-muted-foreground opacity-50" />
            <h3 className="mt-4 text-lg font-medium">Start a conversation</h3>
            <p className="mt-2 text-center text-muted-foreground max-w-md">
              Ask questions about your project, generate code, or get help with
              development tasks. Claude has access to your project context via
              MCP.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {isTyping && (
              <div className="flex items-center gap-2 text-muted-foreground">
                <RefreshCw className="h-4 w-4 animate-spin" />
                <span>Claude is thinking...</span>
              </div>
            )}
          </div>
        )}
      </ScrollArea>

      {/* Error Display */}
      {error && (
        <Card className="mx-4 mb-2 p-3 border-destructive bg-destructive/10">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-destructive">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
            </div>
            <Button variant="ghost" size="sm" onClick={handleClearError}>
              Dismiss
            </Button>
          </div>
        </Card>
      )}

      {/* Input Area */}
      <div className="border-t p-4">
        <div className="flex gap-2">
          <Textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ask Claude about your project..."
            className="min-h-[80px] resize-none"
            disabled={isTyping}
          />
          <Button
            onClick={handleSend}
            disabled={!inputValue.trim() || isTyping}
            className="shrink-0"
          >
            {isTyping ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <Send className="h-4 w-4" />
            )}
          </Button>
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          Press Enter to send, Shift+Enter for new line
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
