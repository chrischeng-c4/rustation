import { useState } from 'react'
import { A2UIRenderer } from './components/A2UIRenderer'
import { PageHeader } from '@/components/shared/PageHeader'
import type { A2UINode, A2UIAction } from './types'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'

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
    <div className="flex h-full flex-col">
      <div className="p-4">
        <PageHeader 
          title="A2UI Renderer" 
          description="Dynamic UI generation from JSON (Simulated Agent Output)"
        />

        <div className="flex gap-6 h-[calc(100vh-200px)]">
          {/* Left: The Rendered UI */}
          <div className="flex-1">
            <ScrollArea className="h-full border rounded-md p-4 bg-slate-50/50">
              <A2UIRenderer node={SAMPLE_PAYLOAD} onAction={handleAction} />
            </ScrollArea>
          </div>

          {/* Right: Debug Info */}
          <div className="w-80 flex flex-col gap-4">
            <Card className="p-4">
              <h3 className="font-semibold mb-2">Last Action</h3>
              <pre className="text-xs font-mono bg-muted p-2 rounded">
                {lastAction ? JSON.stringify(lastAction, null, 2) : 'None'}
              </pre>
            </Card>

            <Card className="p-4 flex-1 overflow-hidden flex flex-col">
              <h3 className="font-semibold mb-2">Source JSON</h3>
              <ScrollArea className="flex-1">
                <pre className="text-xs font-mono text-muted-foreground">
                  {JSON.stringify(SAMPLE_PAYLOAD, null, 2)}
                </pre>
              </ScrollArea>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}
