import { useEffect, useState, useCallback } from 'react'
import { alpha } from '@mui/material/styles'
import { Box, IconButton, Paper, Stack, Typography, useTheme } from '@mui/material'
import { CheckCircle, Close, ErrorOutline, InfoOutlined, WarningAmber } from '@mui/icons-material'
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
 * Get background color for notification type
 */
interface ToastItemProps {
  notification: Notification
  onDismiss: (id: string) => void
}

function ToastItem({ notification, onDismiss }: ToastItemProps) {
  const theme = useTheme()

  const accentColor = (() => {
    switch (notification.notification_type) {
      case 'success':
        return theme.palette.success.main
      case 'error':
        return theme.palette.error.main
      case 'warning':
        return theme.palette.warning.main
      case 'info':
      default:
        return theme.palette.info.main
    }
  })()

  useEffect(() => {
    // Auto-dismiss after timeout
    const timer = setTimeout(() => {
      onDismiss(notification.id)
    }, AUTO_DISMISS_MS)

    return () => clearTimeout(timer)
  }, [notification.id, onDismiss])

  return (
    <Paper
      variant="outlined"
      sx={{
        display: 'flex',
        alignItems: 'flex-start',
        gap: 1.5,
        p: 2,
        borderColor: alpha(accentColor, 0.5),
        bgcolor: alpha(accentColor, 0.08),
        boxShadow: 6,
      }}
    >
      <Box sx={{ flexShrink: 0 }}>{getToastIcon(notification.notification_type)}</Box>
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <Typography variant="body2" fontWeight={500}>
          {notification.message}
        </Typography>
      </Box>
      <IconButton size="small" onClick={() => onDismiss(notification.id)} sx={{ mt: -0.5 }}>
        <Close fontSize="small" />
      </IconButton>
    </Paper>
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
    <Stack spacing={1} sx={{ position: 'fixed', bottom: 16, right: 16, zIndex: 50, maxWidth: 360 }}>
      {displayedToasts.map((notification) => (
        <ToastItem
          key={notification.id}
          notification={notification}
          onDismiss={handleDismiss}
        />
      ))}
    </Stack>
  )
}
