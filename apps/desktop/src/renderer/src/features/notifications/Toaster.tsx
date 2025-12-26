import { useEffect, useState, useCallback } from 'react'
import { X, AlertCircle, AlertTriangle, Info, CheckCircle2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useNotificationsState } from '@/hooks/useAppState'
import type { Notification, NotificationType } from '@/types/state'

/**
 * Auto-dismiss duration in milliseconds
 */
const AUTO_DISMISS_MS = 5000

/**
 * Get icon for notification type
 */
function getToastIcon(type: NotificationType) {
  switch (type) {
    case 'success':
      return <CheckCircle2 className="h-5 w-5 text-green-500" />
    case 'error':
      return <AlertCircle className="h-5 w-5 text-destructive" />
    case 'warning':
      return <AlertTriangle className="h-5 w-5 text-yellow-500" />
    case 'info':
    default:
      return <Info className="h-5 w-5 text-blue-500" />
  }
}

/**
 * Get background color for notification type
 */
function getToastBg(type: NotificationType) {
  switch (type) {
    case 'success':
      return 'border-green-500/50 bg-green-500/10'
    case 'error':
      return 'border-destructive/50 bg-destructive/10'
    case 'warning':
      return 'border-yellow-500/50 bg-yellow-500/10'
    case 'info':
    default:
      return 'border-blue-500/50 bg-blue-500/10'
  }
}

interface ToastItemProps {
  notification: Notification
  onDismiss: (id: string) => void
}

function ToastItem({ notification, onDismiss }: ToastItemProps) {
  useEffect(() => {
    // Auto-dismiss after timeout
    const timer = setTimeout(() => {
      onDismiss(notification.id)
    }, AUTO_DISMISS_MS)

    return () => clearTimeout(timer)
  }, [notification.id, onDismiss])

  return (
    <Card
      className={`flex items-start gap-3 p-4 shadow-lg border animate-in slide-in-from-right-full duration-300 ${getToastBg(notification.notification_type)}`}
    >
      <div className="shrink-0">
        {getToastIcon(notification.notification_type)}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium">{notification.message}</p>
      </div>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0 -mr-1 -mt-1"
        onClick={() => onDismiss(notification.id)}
      >
        <X className="h-4 w-4" />
      </Button>
    </Card>
  )
}

/**
 * Toaster component - Fixed overlay at bottom-right.
 * Shows active (unread) notifications with auto-dismiss.
 */
export function Toaster() {
  const { notifications, dispatch } = useNotificationsState()
  const [visibleIds, setVisibleIds] = useState<Set<string>>(new Set())

  // Track new unread notifications
  useEffect(() => {
    const unreadNotifications = notifications.filter((n) => !n.read)
    const unreadIds = new Set(unreadNotifications.map((n) => n.id))

    // Add new notification IDs to visible set
    setVisibleIds((prev) => {
      const newSet = new Set(prev)
      unreadIds.forEach((id) => newSet.add(id))
      return newSet
    })
  }, [notifications])

  const handleDismiss = useCallback(
    async (id: string) => {
      // Remove from visible toasts
      setVisibleIds((prev) => {
        const newSet = new Set(prev)
        newSet.delete(id)
        return newSet
      })

      // Mark as read in state
      await dispatch({ type: 'MarkNotificationRead', payload: { id } })
    },
    [dispatch]
  )

  // Get visible toasts (unread and in visible set)
  const visibleToasts = notifications.filter(
    (n) => !n.read && visibleIds.has(n.id)
  )

  // Limit to 3 toasts at a time, newest first
  const displayedToasts = [...visibleToasts]
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 3)

  if (displayedToasts.length === 0) {
    return null
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      {displayedToasts.map((notification) => (
        <ToastItem
          key={notification.id}
          notification={notification}
          onDismiss={handleDismiss}
        />
      ))}
    </div>
  )
}
