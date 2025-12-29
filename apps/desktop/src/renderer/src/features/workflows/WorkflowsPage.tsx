import { useState } from 'react'
import { Scroll, ChevronRight, GitBranch } from 'lucide-react'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ConstitutionPanel } from './ConstitutionPanel'
import { ChangeManagementPanel } from './ChangeManagementPanel'

/**
 * Available workflow definitions.
 * In the future, this could be loaded from the backend.
 */
const WORKFLOWS = [
  {
    id: 'constitution-init',
    name: 'Constitution Setup',
    description: 'Initialize or update project constitution for AI-assisted development',
    icon: Scroll,
  },
  {
    id: 'change-management',
    name: 'Change Management',
    description: 'Create and manage changes with proposal and plan generation',
    icon: GitBranch,
  },
]

/**
 * WorkflowsPage - State machine driven guided workflows.
 *
 * Unlike Tasks (simple fire-and-forget justfile commands),
 * Workflows are multi-step, stateful processes that may invoke
 * Claude Code at various nodes.
 */
export function WorkflowsPage() {
  const [selectedWorkflow, setSelectedWorkflow] = useState<string | null>('constitution-init')

  const renderWorkflowPanel = () => {
    switch (selectedWorkflow) {
      case 'constitution-init':
        return <ConstitutionPanel />
      case 'change-management':
        return <ChangeManagementPanel />
      default:
        return (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            Select a workflow to get started
          </div>
        )
    }
  }

  return (
    <div className="flex h-full gap-4">
      {/* Workflow List (Left Column) */}
      <div className="w-72 flex-shrink-0 space-y-2">
        <h2 className="mb-3 text-lg font-semibold">Workflows</h2>
        {WORKFLOWS.map((workflow) => {
          const Icon = workflow.icon
          const isSelected = selectedWorkflow === workflow.id

          return (
            <Card
              key={workflow.id}
              className={`cursor-pointer transition-colors hover:bg-accent ${
                isSelected ? 'border-primary bg-accent' : ''
              }`}
              onClick={() => setSelectedWorkflow(workflow.id)}
            >
              <CardHeader className="p-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Icon className="h-4 w-4 text-muted-foreground" />
                    <CardTitle className="text-sm">{workflow.name}</CardTitle>
                  </div>
                  <ChevronRight
                    className={`h-4 w-4 text-muted-foreground transition-transform ${
                      isSelected ? 'rotate-90' : ''
                    }`}
                  />
                </div>
                <CardDescription className="text-xs">{workflow.description}</CardDescription>
              </CardHeader>
            </Card>
          )
        })}
      </div>

      {/* Workflow Execution Panel (Right Column) */}
      <div className="flex-1">{renderWorkflowPanel()}</div>
    </div>
  )
}
