<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useModelStore } from '@/stores/model'
import ModelDiagram from '@/components/ModelDiagram.vue'
import TimeSeriesChart from '@/components/TimeSeriesChart.vue'
import SimulationControls from '@/components/SimulationControls.vue'
import MetricsPanel from '@/components/MetricsPanel.vue'
import PropertiesPanel from '@/components/PropertiesPanel.vue'
import type { NodeLayout } from '@/types'

const modelStore = useModelStore()
const activeTab = ref<'diagram' | 'simulation'>('diagram')
const selectedNode = ref<NodeLayout | null>(null)

function handleNodeSelect(node: NodeLayout | null) {
  selectedNode.value = node
}

onMounted(async () => {
  try {
    await modelStore.loadModels()

    // Auto-load first model if available
    if (modelStore.models.length > 0) {
      await modelStore.selectModel(modelStore.models[0].id)
    }
  } catch (error) {
    console.error('Failed to initialize dashboard:', error)
  }
})

async function handleUploadModel(file: File) {
  try {
    const model = await modelStore.uploadModel(file)
    await modelStore.selectModel(model.id)
  } catch (error) {
    console.error('Upload failed:', error)
    alert('Failed to upload model. Please check the file format.')
  }
}

async function handleLoadModel() {
  if (modelStore.currentModel) {
    await modelStore.selectModel(modelStore.currentModel.id)
  }
}

function handleStartSimulation() {
  try {
    modelStore.startSimulation()
  } catch (error) {
    console.error('Failed to start simulation:', error)
    alert('Failed to start simulation.')
  }
}

function handleStopSimulation() {
  modelStore.stopSimulation()
}
</script>

<template>
  <div class="dashboard">
    <div class="controls-wrapper">
      <div class="controls-content">
        <SimulationControls
          :has-model="modelStore.hasModel"
          :is-running="modelStore.isSimulationRunning"
          :status="modelStore.simulationStatus"
          @upload-model="handleUploadModel"
          @load-model="handleLoadModel"
          @start-simulation="handleStartSimulation"
          @stop-simulation="handleStopSimulation"
        />
        <div v-if="modelStore.currentModel" class="model-info">
          <strong>{{ modelStore.currentModel.name }}</strong>
          <span v-if="modelStore.layout"> | {{ modelStore.layout.nodes.length }} nodes, {{ modelStore.layout.edges.length }} edges</span>
        </div>
      </div>
    </div>

    <div class="tabs">
      <button
        class="tab"
        :class="{ active: activeTab === 'diagram' }"
        @click="activeTab = 'diagram'"
      >
        📊 Model Diagram
      </button>
      <button
        class="tab"
        :class="{ active: activeTab === 'simulation' }"
        @click="activeTab = 'simulation'"
      >
        📈 Time Series Simulation
      </button>
    </div>

    <div class="tab-content">
      <div v-if="activeTab === 'diagram'" class="diagram-layout">
        <div class="diagram-panel panel">
          <ModelDiagram :layout="modelStore.layout" @select-node="handleNodeSelect" />
        </div>
        <div class="properties-panel-container panel">
          <PropertiesPanel :node="selectedNode" @close="selectedNode = null" />
        </div>
      </div>

      <div v-if="activeTab === 'simulation'" class="panel full-width">
        <TimeSeriesChart :data="modelStore.simulationData" :variables="modelStore.variables" />
      </div>
    </div>

    <div class="metrics-wrapper">
      <MetricsPanel :data="modelStore.simulationData" :variables="modelStore.variables" />
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  width: 100vw;
  height: 100vh;
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  overflow-x: hidden;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.controls-wrapper {
  flex-shrink: 0;
  padding: 10px 15px;
  margin: 0;
  background: white;
  border-bottom: 2px solid #e5e7eb;
  width: 100%;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.controls-content {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.model-info {
  padding: 8px 15px;
  background: #f0f9ff;
  border-left: 3px solid #3b82f6;
  border-radius: 4px;
  font-size: 13px;
  white-space: nowrap;
}

.model-info span {
  color: #555;
  margin-left: 5px;
}

.tabs {
  flex-shrink: 0;
  display: flex;
  gap: 5px;
  padding: 0 15px;
  margin: 0;
  background: #fafafa;
  border-bottom: 2px solid #e5e7eb;
  width: 100%;
}

.tab {
  padding: 12px 24px;
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: #666;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: -2px;
}

.tab:hover {
  color: #3b82f6;
  background: #f0f9ff;
}

.tab.active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
  background: white;
}

.tab-content {
  flex: 1;
  margin: 0;
  position: relative;
  padding: 0;
  box-sizing: border-box;
  width: 100%;
  overflow: hidden;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.diagram-layout {
  display: flex;
  flex: 1;
  gap: 0;
  width: 100%;
  box-sizing: border-box;
  min-height: 0;
}

.diagram-panel {
  width: calc(100vw - 350px);
  height: 100%;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

.properties-panel-container {
  width: 350px;
  max-width: 350px;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
  position: relative;
  border-left: 1px solid #e5e7eb;
}

.panel {
  background: white;
  border-radius: 0;
  padding: 20px;
  box-shadow: none;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.diagram-panel.panel {
  padding: 0;
  border-radius: 0;
  background: #fafafa;
}

.panel.full-width {
  width: 100%;
  height: 600px;
}

@media (max-width: 1200px) {
  .properties-panel-container {
    width: 280px;
    max-width: 280px;
  }
}

.metrics-wrapper {
  flex-shrink: 0;
  display: none;
}

@media (max-width: 768px) {
  .tab {
    padding: 10px 16px;
    font-size: 14px;
  }

  .diagram-layout {
    flex-direction: column;
  }

  .properties-panel-container {
    display: none;
  }
}
</style>
