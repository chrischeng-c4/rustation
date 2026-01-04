export interface A2UIAction {
  type: string
  [key: string]: unknown
}

export interface A2UINode {
  id?: string
  type: string
  props?: Record<string, unknown>
  children?: (A2UINode | string)[]
  content?: string // For simple text nodes
  action?: A2UIAction
}

export interface A2UIRendererProps {
  node: A2UINode
  onAction?: (action: A2UIAction) => void
}
