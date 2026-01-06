import { Badge, Box, IconButton, Tooltip } from '@mui/material'
import { ListAlt, WarningAmber, InfoOutlined, Search, BarChart } from '@mui/icons-material'
import { useAppState } from '@/hooks/useAppState'

type LogPanelType = 'actions' | 'errors' | 'info' | 'debug' | 'metrics'

interface IconBarItemProps {
  icon: React.ReactNode
  label: string
  count: number
  panelType: LogPanelType
  isActive: boolean
  onClick: () => void
}

function IconBarItem({ icon, label, count, isActive, onClick }: IconBarItemProps) {
  return (
    <Tooltip title={`${label} (${count})`} placement="left">
      <IconButton
        onClick={onClick}
        color={isActive ? 'primary' : 'default'}
        sx={{ height: 48, width: 48, position: 'relative' }}
      >
        <Badge
          color={isActive ? 'primary' : 'default'}
          badgeContent={count > 99 ? '99+' : count}
          invisible={count === 0}
          sx={{ '& .MuiBadge-badge': { fontSize: '0.65rem', minWidth: 18, height: 18 } }}
        >
          {icon}
        </Badge>
      </IconButton>
    </Tooltip>
  )
}

export function RightIconBar() {
  const { state, dispatch } = useAppState()

  if (!state) return null

  const { ui_layout, dev_logs } = state
  const activePanel = ui_layout?.active_panel

  // Calculate counts for each panel type
  const counts = {
    actions: dev_logs?.filter((log) => log.log_type === 'action').length ?? 0,
    errors: dev_logs?.filter((log) => log.log_type === 'error').length ?? 0,
    info: dev_logs?.filter((log) => log.log_type === 'info').length ?? 0,
    debug: dev_logs?.length ?? 0, // All logs for debug
    metrics: 0 // Placeholder for future metrics
  }

  const handleTogglePanel = (panelType: LogPanelType) => {
    dispatch({
      type: 'ToggleLogPanel',
      payload: { panel_type: panelType }
    })
  }

  return (
    <Box sx={{ display: 'flex', width: 56, flexDirection: 'column', alignItems: 'center', gap: 1, borderLeft: 1, borderColor: 'divider', bgcolor: 'background.paper', p: 1 }}>
      <IconBarItem
        icon={<ListAlt fontSize="small" />}
        label="Actions"
        count={counts.actions}
        panelType="actions"
        isActive={activePanel === 'actions'}
        onClick={() => handleTogglePanel('actions')}
      />
      <IconBarItem
        icon={<WarningAmber fontSize="small" />}
        label="Errors"
        count={counts.errors}
        panelType="errors"
        isActive={activePanel === 'errors'}
        onClick={() => handleTogglePanel('errors')}
      />
      <IconBarItem
        icon={<InfoOutlined fontSize="small" />}
        label="Info"
        count={counts.info}
        panelType="info"
        isActive={activePanel === 'info'}
        onClick={() => handleTogglePanel('info')}
      />
      <IconBarItem
        icon={<Search fontSize="small" />}
        label="Debug"
        count={counts.debug}
        panelType="debug"
        isActive={activePanel === 'debug'}
        onClick={() => handleTogglePanel('debug')}
      />
      <IconBarItem
        icon={<BarChart fontSize="small" />}
        label="Metrics"
        count={counts.metrics}
        panelType="metrics"
        isActive={activePanel === 'metrics'}
        onClick={() => handleTogglePanel('metrics')}
      />
    </Box>
  )
}
