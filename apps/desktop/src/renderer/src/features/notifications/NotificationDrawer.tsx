import { useCallback } from 'react'
import {
  Bell,
  Check,
  CheckCheck,
  Trash2,
  X,
  AlertCircle,
  AlertTriangle,
  Info,
  CheckCircle2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet'
import { Badge } from '@/components/ui/badge'
import { useNotificationsState } from '@/hooks/useAppState'
import type { Notification, NotificationType } from '@/types/state'

/**
 * Get icon for notification type
 */
function getNotificationIcon(type: NotificationType) {
  switch (type) {
    case 'success':
      return <CheckCircle2 className="h-4 w-4 text-green-500" />
    case 'error':
      return <AlertCircle className="h-4 w-4 text-destructive" />
    case 'warning':
      return <AlertTriangle className="h-4 w-4 text-yellow-500" />
    case 'info':
    default:
      return <Info className="h-4 w-4 text-blue-500" />
  }
}

/**
 * Get badge variant for notification type
 */
function getNotificationBadgeVariant(type: NotificationType) {
  switch (type) {
    case 'success':
      return 'default' as const
    case 'error':
      return 'destructive' as const
    case 'warning':
      return 'secondary' as const
    case 'info':
    default:
      return 'outline' as const
  }
}

/**
 * Format timestamp for display
 */
function formatTimestamp(isoString: string): string {
  const date = new Date(isoString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`

  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })
}

interface NotificationItemProps {
  notification: Notification
  onDismiss: (id: string) => void
  onMarkRead: (id: string) => void
}

function NotificationItem({
  notification,
  onDismiss,
  onMarkRead,
}: NotificationItemProps) {
  return (
    <div
      className={`flex items-start gap-3 p-3 border-b last:border-b-0 ${
        notification.read ? 'opacity-60' : 'bg-muted/30'
      }`}
    >
      {/* Icon */}
      <div className="mt-0.5">
        {getNotificationIcon(notification.notification_type)}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <Badge variant={getNotificationBadgeVariant(notification.notification_type)}>
            {notification.notification_type}
          </Badge>
          {!notification.read && (
            <span className="h-2 w-2 rounded-full bg-blue-500" />
          )}
        </div>
        <p className="text-sm break-words">{notification.message}</p>
        <p className="text-xs text-muted-foreground mt-1">
          {formatTimestamp(notification.created_at)}
        </p>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1">
        {!notification.read && (
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => onMarkRead(notification.id)}
            title="Mark as read"
          >
            <Check className="h-3.5 w-3.5" />
          </Button>
        )}
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onDismiss(notification.id)}
          title="Dismiss"
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  )
}

/**
 * Notification Drawer with bell icon trigger.
 * Shows notification history and unread badge.
 */
export function NotificationDrawer() {
  const { notifications, unreadCount, dispatch } = useNotificationsState()

  const handleDismiss = useCallback(
    async (id: string) => {
      await dispatch({ type: 'DismissNotification', payload: { id } })
    },
    [dispatch]
  )

  const handleMarkRead = useCallback(
    async (id: string) => {
      await dispatch({ type: 'MarkNotificationRead', payload: { id } })
    },
    [dispatch]
  )

  const handleMarkAllRead = useCallback(async () => {
    await dispatch({ type: 'MarkAllNotificationsRead' })
  }, [dispatch])

  const handleClearAll = useCallback(async () => {
    await dispatch({ type: 'ClearNotifications' })
  }, [dispatch])

  // Sort notifications by timestamp (newest first)
  const sortedNotifications = [...notifications].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  )

  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-5 w-5" />
          {unreadCount > 0 && (
            <span className="absolute -top-1 -right-1 h-5 w-5 rounded-full bg-destructive text-destructive-foreground text-xs flex items-center justify-center">
              {unreadCount > 9 ? '9+' : unreadCount}
            </span>
          )}
        </Button>
      </SheetTrigger>
      <SheetContent className="w-[400px] sm:w-[540px]">
        <SheetHeader className="border-b pb-4">
          <div className="flex items-center justify-between">
            <SheetTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notifications
              {unreadCount > 0 && (
                <Badge variant="secondary">{unreadCount} unread</Badge>
              )}
            </SheetTitle>
            <div className="flex items-center gap-2">
              {unreadCount > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleMarkAllRead}
                >
                  <CheckCheck className="mr-2 h-4 w-4" />
                  Mark all read
                </Button>
              )}
              {notifications.length > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleClearAll}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Clear all
                </Button>
              )}
            </div>
          </div>
        </SheetHeader>

        <ScrollArea className="h-[calc(100vh-120px)] mt-4">
          {sortedNotifications.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
              <Bell className="h-12 w-12 opacity-50" />
              <p className="mt-4 text-sm">No notifications</p>
            </div>
          ) : (
            <div className="space-y-0">
              {sortedNotifications.map((notification) => (
                <NotificationItem
                  key={notification.id}
                  notification={notification}
                  onDismiss={handleDismiss}
                  onMarkRead={handleMarkRead}
                />
              ))}
            </div>
          )}
        </ScrollArea>
      </SheetContent>
    </Sheet>
  )
}
