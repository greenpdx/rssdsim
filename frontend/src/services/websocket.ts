import type { SimulationMessage, DataMessage } from '@/types'

export type MessageHandler = (message: SimulationMessage) => void

export class SimulationWebSocket {
  private ws: WebSocket | null = null
  private handlers: MessageHandler[] = []

  connect(modelId: string): void {
    // Use relative path for WebSocket - Vite proxy will handle it
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    this.ws = new WebSocket(`${protocol}//${host}/ws/simulation/${modelId}/`)

    this.ws.onopen = () => {
      console.log('WebSocket connected')
    }

    this.ws.onmessage = (event) => {
      try {
        const message: SimulationMessage = JSON.parse(event.data)
        this.handlers.forEach((handler) => handler(message))
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    this.ws.onclose = () => {
      console.log('WebSocket disconnected')
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  onMessage(handler: MessageHandler): void {
    this.handlers.push(handler)
  }

  removeMessageHandler(handler: MessageHandler): void {
    this.handlers = this.handlers.filter((h) => h !== handler)
  }

  send(message: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
    }
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }
}

export function createWebSocket(): SimulationWebSocket {
  return new SimulationWebSocket()
}
