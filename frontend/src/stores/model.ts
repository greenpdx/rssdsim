import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { modelApi } from '@/services/api'
import { createWebSocket } from '@/services/websocket'
import type {
  ModelInfo,
  LayoutResult,
  SimulationDataPoint,
  SimulationMessage,
  DataMessage,
  CompleteMessage,
  ErrorMessage,
} from '@/types'

export const useModelStore = defineStore('model', () => {
  // State
  const models = ref<ModelInfo[]>([])
  const currentModel = ref<ModelInfo | null>(null)
  const layout = ref<LayoutResult | null>(null)
  const simulationData = ref<SimulationDataPoint[]>([])
  const isSimulationRunning = ref(false)
  const simulationStatus = ref<string>('Ready')
  const maxDataPoints = ref(500)

  // WebSocket instance
  const ws = createWebSocket()

  // Computed
  const hasModel = computed(() => currentModel.value !== null)
  const hasLayout = computed(() => layout.value !== null)
  const variables = computed(() => {
    if (simulationData.value.length === 0) return []
    const firstPoint = simulationData.value[0]
    return Object.keys(firstPoint).filter((key) => key !== 'time')
  })

  // Actions
  async function loadModels() {
    try {
      models.value = await modelApi.listModels()
    } catch (error) {
      console.error('Failed to load models:', error)
      throw error
    }
  }

  async function uploadModel(file: File) {
    try {
      const model = await modelApi.uploadModel(file)
      models.value.push(model)
      return model
    } catch (error) {
      console.error('Failed to upload model:', error)
      throw error
    }
  }

  async function selectModel(modelId: string) {
    try {
      console.log('Loading model:', modelId)
      const model = await modelApi.getModel(modelId)
      console.log('Model info:', model)
      currentModel.value = model

      const layoutData = await modelApi.getModelStructure(modelId)
      console.log('Layout data:', layoutData.nodes.length, 'nodes,', layoutData.edges.length, 'edges')
      layout.value = layoutData

      simulationStatus.value = `Model loaded: ${model.name} (${layoutData.nodes.length} nodes)`
    } catch (error) {
      console.error('Failed to load model:', error)
      throw error
    }
  }

  async function deleteModel(modelId: string) {
    try {
      await modelApi.deleteModel(modelId)
      models.value = models.value.filter((m) => m.id !== modelId)

      if (currentModel.value?.id === modelId) {
        currentModel.value = null
        layout.value = null
      }
    } catch (error) {
      console.error('Failed to delete model:', error)
      throw error
    }
  }

  function startSimulation() {
    if (!currentModel.value) {
      throw new Error('No model selected')
    }

    simulationData.value = []
    isSimulationRunning.value = true
    simulationStatus.value = 'Simulation running...'

    // Set up message handler
    const messageHandler = (message: SimulationMessage) => {
      switch (message.type) {
        case 'start':
          simulationStatus.value = `Simulation started: ${message.model_name}`
          break

        case 'data':
          handleDataMessage(message as DataMessage)
          break

        case 'complete':
          handleCompleteMessage(message as CompleteMessage)
          break

        case 'error':
          handleErrorMessage(message as ErrorMessage)
          break
      }
    }

    ws.onMessage(messageHandler)
    ws.connect(currentModel.value.id)
  }

  function stopSimulation() {
    ws.disconnect()
    isSimulationRunning.value = false
    simulationStatus.value = 'Simulation stopped'
  }

  function handleDataMessage(message: DataMessage) {
    const dataPoint: SimulationDataPoint = {
      time: message.time,
      ...message.values,
    }

    simulationData.value.push(dataPoint)

    // Limit data points
    if (simulationData.value.length > maxDataPoints.value) {
      simulationData.value.shift()
    }
  }

  function handleCompleteMessage(message: CompleteMessage) {
    isSimulationRunning.value = false
    simulationStatus.value = `Simulation complete! (${message.total_steps} steps in ${message.elapsed_ms}ms)`
    ws.disconnect()
  }

  function handleErrorMessage(message: ErrorMessage) {
    isSimulationRunning.value = false
    simulationStatus.value = `Error: ${message.message}`
    ws.disconnect()
  }

  function clearSimulationData() {
    simulationData.value = []
  }

  return {
    // State
    models,
    currentModel,
    layout,
    simulationData,
    isSimulationRunning,
    simulationStatus,
    maxDataPoints,

    // Computed
    hasModel,
    hasLayout,
    variables,

    // Actions
    loadModels,
    uploadModel,
    selectModel,
    deleteModel,
    startSimulation,
    stopSimulation,
    clearSimulationData,
  }
})
