<script setup lang="ts">
import { computed } from 'vue'
import type { SimulationDataPoint } from '@/types'

const props = defineProps<{
  data: SimulationDataPoint[]
  variables: string[]
}>()

const latestData = computed(() => {
  if (props.data.length === 0) return null
  return props.data[props.data.length - 1]
})

const currentTime = computed(() => {
  return latestData.value?.time.toFixed(2) || '0.00'
})

function getCurrentValue(varName: string): string {
  if (!latestData.value) return '-'
  const value = latestData.value[varName]
  if (value === undefined) return '-'
  return value.toFixed(2)
}
</script>

<template>
  <div class="metrics-panel">
    <h3>Live Metrics</h3>
    <div class="metrics-grid">
      <div class="metric">
        <div class="metric-label">Time</div>
        <div class="metric-value">{{ currentTime }}</div>
      </div>

      <div v-for="varName in variables" :key="varName" class="metric">
        <div class="metric-label">{{ varName }}</div>
        <div class="metric-value">{{ getCurrentValue(varName) }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.metrics-panel {
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

h3 {
  margin: 0 0 15px 0;
  font-size: 18px;
  color: #333;
  border-bottom: 2px solid #3b82f6;
  padding-bottom: 10px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 15px;
}

.metric {
  background: #f9fafb;
  padding: 15px;
  border-radius: 4px;
  text-align: center;
}

.metric-label {
  font-size: 12px;
  color: #666;
  margin-bottom: 5px;
}

.metric-value {
  font-size: 24px;
  font-weight: bold;
  color: #333;
}
</style>
