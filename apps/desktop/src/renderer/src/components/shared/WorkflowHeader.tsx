import React from 'react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface WorkflowHeaderProps {
  title: string
  subtitle?: string
  status?: string
  statusColor?: string
  icon?: React.ReactNode
  children?: React.ReactNode
  className?: string
}

/**
 * WorkflowHeader - Standardized header for workflow sub-panels.
 * Smaller and more compact than PageHeader, designed for nested views.
 */
export function WorkflowHeader({
  title,
  subtitle,
  status,
  statusColor = "bg-blue-500",
  icon,
  children,
  className,
}: WorkflowHeaderProps) {
  return (
    <div className={cn("flex items-center justify-between border-b bg-muted/40 px-4 py-2 h-12 shrink-0", className)}>
      <div className="flex items-center gap-3 overflow-hidden">
        {icon && <div className="flex-shrink-0">{icon}</div>}
        <div className="flex flex-col min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-semibold truncate">{title}</h3>
            {status && (
              <Badge variant="secondary" className={cn("h-4 px-1.5 text-[10px] text-white border-none", statusColor)}>
                {status}
              </Badge>
            )}
          </div>
          {subtitle && (
            <p className="text-[10px] text-muted-foreground truncate">
              {subtitle}
            </p>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2 flex-shrink-0">
        {children}
      </div>
    </div>
  )
}
