export interface JustCommand {
  name: string
  description: string | null
  recipe: string
}

export type TaskStatus = 'idle' | 'running' | 'success' | 'error'

export interface TaskState {
  commands: JustCommand[]
  runningCommand: string | null
  lastOutput: string | null
  lastError: string | null
}
