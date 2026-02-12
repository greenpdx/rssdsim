export interface ModelInfo {
  id: string
  name: string
  created_at: number
  stocks_count: number
  flows_count: number
}

export interface NodeLayout {
  id: string
  label: string
  x: number
  y: number
  width: number
  height: number
  type: NodeType
  metadata?: {
    initial_value?: number
    equation?: string
    units?: string
  }
}

export interface EdgeLayout {
  from: string
  to: string
  edge_type: EdgeType
}

export interface LayoutResult {
  nodes: NodeLayout[]
  edges: EdgeLayout[]
  width: number
  height: number
}

export type NodeType = 'stock' | 'flow' | 'auxiliary' | 'parameter'
export type EdgeType = 'flow' | 'dependency'

export interface TimeConfig {
  start: number
  stop: number
  dt: number
  save_step: number
}

export interface WebSocketMessage {
  type: 'start' | 'data' | 'complete' | 'error'
}

export interface StartMessage extends WebSocketMessage {
  type: 'start'
  model_name: string
  variables: string[]
  time_config: TimeConfig
}

export interface DataMessage extends WebSocketMessage {
  type: 'data'
  time: number
  values: Record<string, number>
}

export interface CompleteMessage extends WebSocketMessage {
  type: 'complete'
  total_steps: number
  elapsed_ms: number
}

export interface ErrorMessage extends WebSocketMessage {
  type: 'error'
  message: string
}

export type SimulationMessage = StartMessage | DataMessage | CompleteMessage | ErrorMessage

export interface SimulationDataPoint {
  time: number
  [key: string]: number
}
