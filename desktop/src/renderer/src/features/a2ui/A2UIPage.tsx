import { useState } from 'react'
import { A2UIRenderer } from './components/A2UIRenderer'
import { PageHeader } from '@/components/shared/PageHeader'
import type { A2UINode, A2UIAction } from './types'
import {
  Box,
  Typography,
  Paper,
  Stack,
  Divider
} from '@mui/material'
import { Code as CodeIcon, BugReport as DebugIcon } from '@mui/icons-material'

// Example Payload simulating what an Agent via MCP might send
const SAMPLE_PAYLOAD: A2UINode = {
  type: 'div',
  props: { className: 'space-y-4 max-w-2xl mx-auto' },
  children: [
    {
      type: 'card',
      children: [
        {
          type: 'card-header',
          children: [
            { type: 'card-title', children: ['Refactoring Plan: User Authentication'] },
            { type: 'card-description', children: ['Proposed changes to auth module based on A2UI review.'] }
          ]
        },
        {
          type: 'card-content',
          props: { className: 'space-y-4' },
          children: [
            {
              type: 'alert',
              props: { variant: 'default', className: 'bg-blue-50 border-blue-200' },
              children: [
                { type: 'icon-info', props: { className: 'h-4 w-4 text-blue-500' } },
                { type: 'alert-title', children: ['Analysis Complete'] },
                { type: 'alert-description', children: ['Found 3 potential security issues in login.rs'] }
              ]
            },
            {
              type: 'div',
              props: { className: 'grid gap-2' },
              children: [
                { type: 'label', children: ['Confirm Branch Name'] },
                { 
                  type: 'input', 
                  props: { defaultValue: 'feat/auth-hardening', placeholder: 'Enter branch name' } 
                }
              ]
            },
            {
              type: 'div',
              props: { className: 'space-y-2 mt-4' },
              children: [
                { type: 'label', props: { className: 'text-xs' }, children: ['Migration Progress'] },
                { type: 'progress', props: { value: 65, className: 'h-2' } }
              ]
            },
            {
              type: 'accordion',
              props: { type: 'single', collapsible: true, className: 'mt-4' },
              children: [
                {
                  type: 'accordion-item',
                  props: { value: 'item-1' },
                  children: [
                    { type: 'accordion-trigger', children: ['Affected Files'] },
                    { 
                      type: 'accordion-content', 
                      children: [
                        { type: 'p', props: { className: 'text-xs' }, children: ['- src/auth/login.rs'] },
                        { type: 'p', props: { className: 'text-xs' }, children: ['- src/auth/token.rs'] }
                      ] 
                    }
                  ]
                }
              ]
            }
          ]
        },
        {
          type: 'card-footer',
          props: { className: 'flex justify-end gap-2' },
          children: [
            {
              type: 'button',
              props: { variant: 'outline' },
              children: ['Reject'],
              action: { type: 'mcp:reject_plan' }
            },
            {
              type: 'button',
              props: { variant: 'default' },
              children: ['Approve & Execute'],
              action: { type: 'mcp:execute_plan', planId: '123' }
            }
          ]
        }
      ]
    }
  ]
}

export function A2UIPage() {
  const [lastAction, setLastAction] = useState<A2UIAction | null>(null)

  const handleAction = (action: A2UIAction) => {
    console.log('A2UI Action:', action)
    setLastAction(action)
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column', p: 3 }}>
      <PageHeader 
        title="A2UI Renderer" 
        description="Dynamic UI generation from JSON (Simulated Agent Output)"
        icon={<CodeIcon />}
      />

      <Stack direction="row" spacing={3} sx={{ flex: 1, minHeight: 0 }}>
        {/* Left: The Rendered UI */}
        <Paper 
          variant="outlined" 
          sx={{ 
            flex: 1, 
            p: 4, 
            bgcolor: 'surfaceContainerLow.main', 
            borderRadius: 4,
            overflow: 'auto' 
          }}
        >
          <A2UIRenderer node={SAMPLE_PAYLOAD} onAction={handleAction} />
        </Paper>

        {/* Right: Debug Info */}
        <Stack spacing={3} sx={{ width: 360, flexShrink: 0 }}>
          <Card variant="outlined" sx={{ borderRadius: 4 }}>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'outlineVariant' }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <DebugIcon fontSize="small" color="primary" />
                <Typography variant="subtitle2" fontWeight={600}>Last Action</Typography>
              </Stack>
            </Box>
            <Box sx={{ p: 2 }}>
              <Box 
                component="pre" 
                sx={{ 
                  m: 0, 
                  p: 1.5, 
                  bgcolor: 'background.default', 
                  borderRadius: 1, 
                  fontSize: '0.7rem', 
                  fontFamily: 'monospace',
                  overflowX: 'auto',
                  border: 1,
                  borderColor: 'outlineVariant'
                }}
              >
                {lastAction ? JSON.stringify(lastAction, null, 2) : 'None'}
              </Box>
            </Box>
          </Card>

          <Card variant="outlined" sx={{ flex: 1, borderRadius: 4, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'outlineVariant' }}>
              <Typography variant="subtitle2" fontWeight={600}>Source JSON</Typography>
            </Box>
            <Box sx={{ flex: 1, p: 2, overflow: 'auto' }}>
              <Box 
                component="pre" 
                sx={{ 
                  m: 0, 
                  p: 1.5, 
                  bgcolor: 'background.default', 
                  borderRadius: 1, 
                  fontSize: '0.7rem', 
                  fontFamily: 'monospace',
                  color: 'onSurfaceVariant.main',
                  border: 1,
                  borderColor: 'outlineVariant'
                }}
              >
                {JSON.stringify(SAMPLE_PAYLOAD, null, 2)}
              </Box>
            </Box>
          </Card>
        </Stack>
      </Stack>
    </Box>
  )
}
