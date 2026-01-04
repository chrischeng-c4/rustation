import React from 'react'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'

interface PageHeaderProps {
  title: string
  description?: string
  children?: React.ReactNode
  className?: string
  icon?: React.ReactNode
}

/**
 * PageHeader - Standardized header for feature pages.
 * Includes title, description, and an action area for buttons.
 */
export function PageHeader({
  title,
  description,
  children,
  className,
  icon,
}: PageHeaderProps) {
  return (
    <div className={cn("mb-6 flex flex-col gap-4", className)}>
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            {icon && <div className="text-muted-foreground">{icon}</div>}
            <h2 className="text-2xl font-semibold tracking-tight">{title}</h2>
          </div>
          {description && (
            <p className="text-sm text-muted-foreground">
              {description}
            </p>
          )}
        </div>
        <div className="flex items-center gap-2">
          {children}
        </div>
      </div>
      <Separator />
    </div>
  )
}
