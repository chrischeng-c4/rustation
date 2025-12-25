import { RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
} from '@/components/ui/sheet'

interface DockerLogSheetProps {
  open: boolean
  serviceName: string
  logs: string[]
  onClose: () => void
  onRefresh?: () => void
}

export function DockerLogSheet({
  open,
  serviceName,
  logs,
  onClose,
  onRefresh,
}: DockerLogSheetProps) {
  return (
    <Sheet open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <SheetContent side="right" className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>{serviceName} Logs</SheetTitle>
          <SheetDescription>Container output logs</SheetDescription>
        </SheetHeader>

        <ScrollArea className="my-4 h-[calc(100vh-200px)]">
          <div className="space-y-1 font-mono text-xs">
            {logs.length > 0 ? (
              logs.map((line, index) => (
                <div key={index} className="whitespace-pre-wrap break-all">
                  {line}
                </div>
              ))
            ) : (
              <p className="text-muted-foreground">No logs available</p>
            )}
          </div>
        </ScrollArea>

        <SheetFooter>
          <Button variant="outline" size="sm" onClick={onRefresh}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
