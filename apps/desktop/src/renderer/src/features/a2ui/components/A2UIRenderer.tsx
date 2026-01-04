import React, { createElement } from 'react'
import { A2UI_REGISTRY } from '../registry'
import type { A2UINode, A2UIAction } from '../types'

interface A2UIRendererProps {
  node: A2UINode
  onAction?: (action: A2UIAction) => void
  depth?: number
}

export function A2UIRenderer({ node, onAction, depth = 0 }: A2UIRendererProps): React.ReactNode {
  // 1. Handle simple text content
  if (typeof node === 'string') {
    return node
  }
  
  if (!node.type) {
    return null
  }

  // 2. Resolve Component
  // Special case: 'text' type is just a span with content
  if (node.type === 'text') {
    return <span>{node.content}</span>
  }

  const Component = A2UI_REGISTRY[node.type]

  if (!Component) {
    console.warn(`A2UI: Unknown component type "${node.type}"`)
    return <div className="text-destructive text-xs border border-destructive p-1">Unknown: {node.type}</div>
  }

  // 3. Prepare Props
  const props: Record<string, any> = { ...node.props }
  
  // Handle actions (e.g., button click)
  if (node.action && onAction) {
    const originalOnClick = props.onClick
    props.onClick = (e: React.MouseEvent) => {
      e.stopPropagation()
      originalOnClick?.(e)
      onAction(node.action!)
    }
  }

  // 4. Render Children (Recursive)
  const children = node.children?.map((child, index) => {
    // Handle string children
    if (typeof child === 'string') {
      return child
    }
    // Handle nested node
    return (
      <A2UIRenderer 
        key={child.id || index} 
        node={child} 
        onAction={onAction}
        depth={depth + 1}
      />
    )
  })

  // If node has 'content' prop, treat it as a child
  if (node.content) {
    children?.push(node.content)
  }

  // 5. Create Element
  return createElement(Component, props, children)
}
