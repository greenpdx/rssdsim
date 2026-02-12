<script setup lang="ts">
import type { NodeLayout } from '@/types'

const props = defineProps<{
  node: NodeLayout | null
}>()

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <div class="properties-panel">
    <div class="panel-header">
      <h3>Properties</h3>
      <button v-if="node" class="close-btn" @click="emit('close')" title="Clear Selection">✕</button>
    </div>

    <div v-if="node" class="panel-content">
      <div class="property-group">
        <div class="property-label">ID</div>
        <div class="property-value">{{ node.id }}</div>
      </div>

      <div class="property-group">
        <div class="property-label">Label</div>
        <div class="property-value">{{ node.label || node.id }}</div>
      </div>

      <div class="property-group">
        <div class="property-label">Type</div>
        <div class="property-value">
          <span class="type-badge" :class="node.type">{{ node.type }}</span>
        </div>
      </div>

      <div class="property-group">
        <div class="property-label">Position</div>
        <div class="property-value">
          x: {{ Math.round(node.x) }}, y: {{ Math.round(node.y) }}
        </div>
      </div>

      <div class="property-group">
        <div class="property-label">Size</div>
        <div class="property-value">
          {{ Math.round(node.width) }} × {{ Math.round(node.height) }}
        </div>
      </div>

      <div v-if="node.metadata?.initial_value !== undefined" class="property-group">
        <div class="property-label">Initial Value</div>
        <div class="property-value code">{{ node.metadata.initial_value }}</div>
      </div>

      <div v-if="node.metadata?.equation" class="property-group">
        <div class="property-label">Equation</div>
        <div class="property-value code">{{ node.metadata.equation }}</div>
      </div>

      <div v-if="node.metadata?.units" class="property-group">
        <div class="property-label">Units</div>
        <div class="property-value">{{ node.metadata.units }}</div>
      </div>
    </div>

    <div v-else class="panel-content empty">
      <div class="empty-state">
        <div class="empty-icon">🖱️</div>
        <div class="empty-text">Click on a node to view its properties</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.properties-panel {
  width: 100%;
  height: 100%;
  background: white;
  display: flex;
  flex-direction: column;
}

.panel-header {
  padding: 15px 20px;
  border-bottom: 2px solid #e5e7eb;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-header h3 {
  margin: 0;
  font-size: 18px;
  color: #333;
}

.close-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: #f3f4f6;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  color: #666;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #fee2e2;
  color: #ef4444;
}

.panel-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.panel-content.empty {
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state {
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 10px;
  opacity: 0.3;
}

.empty-text {
  color: #9ca3af;
  font-size: 14px;
}

.property-group {
  margin-bottom: 20px;
}

.property-label {
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.property-value {
  font-size: 14px;
  color: #1f2937;
  word-break: break-word;
}

.property-value.code {
  font-family: 'Courier New', monospace;
  background: #f9fafb;
  padding: 8px 12px;
  border-radius: 4px;
  border: 1px solid #e5e7eb;
  font-size: 13px;
}

.type-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  text-transform: capitalize;
}

.type-badge.stock {
  background: #dbeafe;
  color: #1e40af;
}

.type-badge.flow {
  background: #fef3c7;
  color: #d97706;
}

.type-badge.auxiliary {
  background: #d1fae5;
  color: #059669;
}

.type-badge.parameter {
  background: #ede9fe;
  color: #7c3aed;
}
</style>
