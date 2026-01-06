import { useCallback, useState } from 'react'
import { alpha } from '@mui/material/styles'
import {
  Badge,
  Box,
  Button,
  Drawer,
  IconButton,
  Stack,
  Typography,
} from '@mui/material'
import {
  Notifications,
  Done,
  DoneAll,
  DeleteOutline,
  Close,
  ErrorOutline,
  WarningAmber,
  InfoOutlined,
  CheckCircle,
} from '@mui/icons-material'
import { useNotificationsState } from '@/hooks/useAppState'
import type { Notification, NotificationType } from '@/types/state'

/**
 * Get icon for notification type
 */
function getNotificationIcon(type: NotificationType) {
  switch (type) {
    case 'success':
      return <CheckCircle fontSize="small" color="success" />
    case 'error':
      return <ErrorOutline fontSize="small" color="error" />
    case 'warning':
      return <WarningAmber fontSize="small" color="warning" />
    case 'info':
    default:
      return <InfoOutlined fontSize="small" color="info" />
  }
}

/**
 * Get badge variant for notification type
 */
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
    <Stack
      direction="row"
      spacing={1.5}
      alignItems="flex-start"
      sx={{
        p: 1.5,
        borderBottom: 1,
        borderColor: 'divider',
        bgcolor: notification.read ? 'transparent' : alpha('#90caf9', 0.08),
        opacity: notification.read ? 0.6 : 1,
      }}
    >
      {/* Icon */}
      <Box sx={{ mt: 0.5 }}>{getNotificationIcon(notification.notification_type)}</Box>

      {/* Content */}
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 0.5 }}>
          <Badge
            color="primary"
            badgeContent={notification.notification_type}
            sx={{
              '& .MuiBadge-badge': {
                position: 'static',
                transform: 'none',
                borderRadius: 8,
                px: 1,
                fontSize: '0.6rem',
                textTransform: 'uppercase',
              },
            }}
          />
          {!notification.read && (
            <Box sx={{ width: 8, height: 8, borderRadius: '50%', bgcolor: 'info.main' }} />
          )}
        </Stack>
        <Typography variant="body2" sx={{ wordBreak: 'break-word' }}>
          {notification.message}
        </Typography>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
          {formatTimestamp(notification.created_at)}
        </Typography>
      </Box>

      {/* Actions */}
      <Stack direction="row" spacing={0.5}>
        {!notification.read && (
          <IconButton size="small" onClick={() => onMarkRead(notification.id)} title="Mark as read">
            <Done fontSize="small" />
          </IconButton>
        )}
        <IconButton size="small" onClick={() => onDismiss(notification.id)} title="Dismiss">
          <Close fontSize="small" />
        </IconButton>
      </Stack>
    </Stack>
  )
}

/**
 * Notification Drawer with bell icon trigger.
 * Shows notification history and unread badge.
 */
export function NotificationDrawer() {
  const { notifications, unreadCount, dispatch } = useNotificationsState()
  const [open, setOpen] = useState(false)

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
    <>
      <IconButton onClick={() => setOpen(true)} title="Notifications">
        <Badge badgeContent={unreadCount > 9 ? '9+' : unreadCount} color="error" invisible={unreadCount === 0}>
          <Notifications fontSize="small" />
        </Badge>
      </IconButton>
      <Drawer anchor="right" open={open} onClose={() => setOpen(false)}>
        <Box sx={{ width: { xs: 360, sm: 540 }, display: 'flex', flexDirection: 'column', height: '100%' }}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ borderBottom: 1, borderColor: 'divider', px: 2, py: 2 }}>
            <Stack direction="row" alignItems="center" spacing={1}>
              <Notifications fontSize="small" />
              <Typography variant="subtitle1" fontWeight={600}>
                Notifications
              </Typography>
              {unreadCount > 0 && (
                <Badge
                  badgeContent={`${unreadCount} unread`}
                  color="primary"
                  sx={{
                    '& .MuiBadge-badge': {
                      position: 'static',
                      transform: 'none',
                      borderRadius: 8,
                      px: 1,
                    },
                  }}
                />
              )}
            </Stack>
            <Stack direction="row" spacing={1}>
              {unreadCount > 0 && (
                <Button variant="outlined" size="small" onClick={handleMarkAllRead} startIcon={<DoneAll fontSize="small" />}>
                  Mark all read
                </Button>
              )}
              {notifications.length > 0 && (
                <Button variant="outlined" size="small" onClick={handleClearAll} startIcon={<DeleteOutline fontSize="small" />}>
                  Clear all
                </Button>
              )}
            </Stack>
          </Stack>

          <Box sx={{ flex: 1, overflow: 'auto', mt: 2 }}>
            {sortedNotifications.length === 0 ? (
              <Stack alignItems="center" justifyContent="center" sx={{ py: 6, color: 'text.secondary' }}>
                <Notifications sx={{ fontSize: 48, opacity: 0.5 }} />
                <Typography variant="body2" sx={{ mt: 2 }}>
                  No notifications
                </Typography>
              </Stack>
            ) : (
              <Box>
                {sortedNotifications.map((notification) => (
                  <NotificationItem
                    key={notification.id}
                    notification={notification}
                    onDismiss={handleDismiss}
                    onMarkRead={handleMarkRead}
                  />
                ))}
              </Box>
            )}
          </Box>
        </Box>
      </Drawer>
    </>
  )
}
