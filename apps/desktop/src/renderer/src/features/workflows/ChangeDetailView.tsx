import { FileText, Play, Check, X, Clock } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useAppState } from '@/hooks/useAppState'
import type { Change } from '@/types/state'

interface ChangeDetailViewProps {
  change: Change
}

/**
 * ChangeDetailView - Shows change details, proposal, and plan
 */
export function ChangeDetailView({ change }: ChangeDetailViewProps) {
  const { dispatch } = useAppState()

  const handleGenerateProposal = () => {
    dispatch({ type: 'GenerateProposal', payload: { change_id: change.id } })
  }

  const handleGeneratePlan = () => {
    dispatch({ type: 'GeneratePlan', payload: { change_id: change.id } })
  }

  const handleApprovePlan = () => {
    dispatch({ type: 'ApprovePlan', payload: { change_id: change.id } })
  }

  const handleCancelChange = () => {
    dispatch({ type: 'CancelChange', payload: { change_id: change.id } })
  }

  const isPlanning = change.status === 'planning'
  const hasProposal = !!change.proposal
  const hasPlan = !!change.plan
  const canGenerateProposal = change.status === 'proposed' && !hasProposal
  const canGeneratePlan = hasProposal && !hasPlan && change.status !== 'planning'
  const canApprove = change.status === 'planned'
  const canCancel = !['done', 'archived', 'cancelled'].includes(change.status)

  return (
    <Card className="h-full">
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-lg">{change.name}</CardTitle>
            <CardDescription className="mt-1">{change.intent}</CardDescription>
          </div>
          <Badge variant="outline" className="capitalize">
            {change.status}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="h-[calc(100%-100px)]">
        <Tabs defaultValue="proposal" className="h-full">
          <TabsList className="mb-4">
            <TabsTrigger value="proposal" className="gap-1">
              <FileText className="h-4 w-4" />
              Proposal
            </TabsTrigger>
            <TabsTrigger value="plan" className="gap-1">
              <FileText className="h-4 w-4" />
              Plan
            </TabsTrigger>
          </TabsList>

          <TabsContent value="proposal" className="h-[calc(100%-50px)]">
            {isPlanning && change.streaming_output ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-yellow-600">
                  <Clock className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Generating...</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : hasProposal ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <pre className="whitespace-pre-wrap text-sm">{change.proposal}</pre>
              </ScrollArea>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-4">
                <FileText className="h-12 w-12 text-muted-foreground" />
                <p className="text-muted-foreground">No proposal generated yet</p>
                {canGenerateProposal && (
                  <Button onClick={handleGenerateProposal}>
                    <Play className="mr-2 h-4 w-4" />
                    Generate Proposal
                  </Button>
                )}
              </div>
            )}
          </TabsContent>

          <TabsContent value="plan" className="h-[calc(100%-50px)]">
            {isPlanning && !hasProposal && change.streaming_output ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <div className="flex items-center gap-2 mb-2 text-yellow-600">
                  <Clock className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Generating plan...</span>
                </div>
                <pre className="whitespace-pre-wrap text-sm">{change.streaming_output}</pre>
              </ScrollArea>
            ) : hasPlan ? (
              <ScrollArea className="h-full rounded-md border p-4">
                <pre className="whitespace-pre-wrap text-sm">{change.plan}</pre>
              </ScrollArea>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-4">
                <FileText className="h-12 w-12 text-muted-foreground" />
                <p className="text-muted-foreground">
                  {hasProposal ? 'No plan generated yet' : 'Generate a proposal first'}
                </p>
                {canGeneratePlan && (
                  <Button onClick={handleGeneratePlan}>
                    <Play className="mr-2 h-4 w-4" />
                    Generate Plan
                  </Button>
                )}
              </div>
            )}
          </TabsContent>
        </Tabs>

        {/* Action Buttons */}
        <div className="mt-4 flex gap-2 border-t pt-4">
          {canApprove && (
            <Button onClick={handleApprovePlan} className="bg-green-600 hover:bg-green-700">
              <Check className="mr-2 h-4 w-4" />
              Approve Plan
            </Button>
          )}
          {canCancel && (
            <Button variant="destructive" onClick={handleCancelChange}>
              <X className="mr-2 h-4 w-4" />
              Cancel Change
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
