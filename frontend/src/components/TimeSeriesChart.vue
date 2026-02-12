<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { Chart, registerables } from 'chart.js'
import type { SimulationDataPoint } from '@/types'

Chart.register(...registerables)

const props = defineProps<{
  data: SimulationDataPoint[]
  variables: string[]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
let chartInstance: Chart | null = null

const colors = [
  '#3b82f6',
  '#ef4444',
  '#10b981',
  '#f59e0b',
  '#8b5cf6',
  '#ec4899',
  '#06b6d4',
  '#84cc16',
]

function createChart() {
  if (!canvasRef.value) return

  // Destroy existing chart
  if (chartInstance) {
    chartInstance.destroy()
  }

  const ctx = canvasRef.value.getContext('2d')
  if (!ctx) return

  const datasets = props.variables.map((varName, index) => ({
    label: varName,
    data: props.data.map((point) => ({ x: point.time, y: point[varName] || 0 })),
    borderColor: colors[index % colors.length],
    backgroundColor: colors[index % colors.length] + '20',
    borderWidth: 2,
    fill: false,
    tension: 0.1,
  }))

  chartInstance = new Chart(ctx, {
    type: 'line',
    data: {
      datasets,
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: {
        duration: 0, // Disable animations for real-time updates
      },
      scales: {
        x: {
          type: 'linear',
          title: {
            display: true,
            text: 'Time',
          },
        },
        y: {
          title: {
            display: true,
            text: 'Value',
          },
        },
      },
      plugins: {
        legend: {
          display: true,
          position: 'top',
        },
        tooltip: {
          mode: 'index',
          intersect: false,
        },
      },
    },
  })
}

function updateChart() {
  if (!chartInstance) return

  const datasets = props.variables.map((varName, index) => ({
    label: varName,
    data: props.data.map((point) => ({ x: point.time, y: point[varName] || 0 })),
    borderColor: colors[index % colors.length],
    backgroundColor: colors[index % colors.length] + '20',
    borderWidth: 2,
    fill: false,
    tension: 0.1,
  }))

  chartInstance.data.datasets = datasets
  chartInstance.update('none') // Update without animation
}

// Watch for data changes
watch(
  () => props.data,
  () => {
    if (chartInstance) {
      updateChart()
    } else {
      createChart()
    }
  },
  { deep: true }
)

watch(
  () => props.variables,
  () => {
    createChart()
  }
)

onMounted(() => {
  createChart()
})
</script>

<template>
  <div class="chart-container">
    <canvas ref="canvasRef"></canvas>
  </div>
</template>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  flex: 1;
  min-height: 500px;
  position: relative;
}
</style>
