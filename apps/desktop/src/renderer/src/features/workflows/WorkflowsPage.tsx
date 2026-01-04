import { useState } from 'react'
import { Scroll, ChevronRight, GitBranch, BookOpen, Workflow } from 'lucide-react'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { EmptyState } from '@/components/shared/EmptyState'
import { ConstitutionPanel } from './ConstitutionPanel'
import { ChangeManagementPanel } from './ChangeManagementPanel'
import { ContextPanel } from './ContextPanel'

/**
 * Available workflow definitions.
 * ReviewGate is NOT a separate workflow - it's integrated into Change Management.
 */
const WORKFLOWS = [
  {
    id: 'constitution-init',
    name: 'Constitution Setup',
    description: 'Initialize or update project constitution for AI-assisted development',
    icon: Scroll,
  },
  {
    id: 'living-context',
    name: 'Living Context',
    description: 'View and manage project context - tech stack, architecture, recent changes',
    icon: BookOpen,
  },
  {
    id: 'change-management',
    name: 'Change Management',
    description: 'Create and manage changes with proposal, plan generation, and review',
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
      case 'living-context':
        return <ContextPanel />
      case 'change-management':
        return <ChangeManagementPanel />
      default:
        return (
          <EmptyState
            icon={Workflow}
            title="Select a Workflow"
            description="Choose a workflow from the list on the left to begin."
          />
        )
    }
  }

  return (
    <div className="flex h-full gap-4">
      {/* Workflow List (Left Column) */}
      <div className="w-64 flex-shrink-0 space-y-2 overflow-y-auto">
        <h2 className="mb-3 text-lg font-semibold px-1">Workflows</h2>
        {WORKFLOWS.map((workflow) => {
          const Icon = workflow.icon
          const isSelected = selectedWorkflow === workflow.id

          return (
            <Card
              key={workflow.id}
              className={`cursor-pointer transition-colors hover:bg-accent ${
                isSelected ? 'border-primary bg-accent shadow-sm' : ''
              }`}
              onClick={() => setSelectedWorkflow(workflow.id)}
            >
              <CardHeader className="p-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Icon className={`h-4 w-4 ${isSelected ? 'text-primary' : 'text-muted-foreground'}`} />
                    <CardTitle className="text-sm">{workflow.name}</CardTitle>
                  </div>
                  <ChevronRight
                    className={`h-4 w-4 text-muted-foreground transition-transform ${
                      isSelected ? 'rotate-90 text-primary' : ''
                    }`}
                  />
                </div>
                <CardDescription className="text-xs line-clamp-2">{workflow.description}</CardDescription>
              </CardHeader>
            </Card>
          )
        })}
      </div>

      {/* Workflow Execution Panel (Right Column) */}
      <div className="flex-1 overflow-hidden">
        <Card className="h-full flex flex-col overflow-hidden">
          {renderWorkflowPanel()}
        </Card>
      </div>
    </div>
  )
}
