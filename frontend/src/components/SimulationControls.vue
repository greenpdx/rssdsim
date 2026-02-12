<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  hasModel: boolean
  isRunning: boolean
  status: string
}>()

const emit = defineEmits<{
  uploadModel: [file: File]
  loadModel: []
  startSimulation: []
  stopSimulation: []
}>()

const fileInputRef = ref<HTMLInputElement | null>(null)

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]

  if (file) {
    emit('uploadModel', file)
    // Reset input
    if (fileInputRef.value) {
      fileInputRef.value.value = ''
    }
  }
}

function triggerFileInput() {
  fileInputRef.value?.click()
}
</script>

<template>
  <div class="controls-container">
    <div class="button-group">
      <input
        ref="fileInputRef"
        type="file"
        accept=".xmile,.xml,.json"
        style="display: none"
        @change="handleFileChange"
      />

      <button class="btn btn-primary" @click="triggerFileInput">📂 Upload Model</button>

      <button class="btn btn-secondary" :disabled="!hasModel" @click="$emit('loadModel')">
        🔄 Reload
      </button>

      <button
        class="btn btn-success"
        :disabled="!hasModel || isRunning"
        @click="$emit('startSimulation')"
      >
        ▶️ Start Simulation
      </button>

      <button class="btn btn-danger" :disabled="!isRunning" @click="$emit('stopSimulation')">
        ⏹️ Stop
      </button>
    </div>

    <div class="status-bar" :class="{ running: isRunning }">
      {{ status }}
    </div>
  </div>
</template>

<style scoped>
.controls-container {
  display: flex;
  flex-direction: column;
  gap: 15px;
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.button-group {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-secondary {
  background: #6b7280;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #4b5563;
}

.btn-success {
  background: #10b981;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background: #059669;
}

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.status-bar {
  padding: 10px;
  background: #f0f9ff;
  border-left: 4px solid #3b82f6;
  font-size: 14px;
  border-radius: 4px;
}

.status-bar.running {
  background: #fef3c7;
  border-left-color: #f59e0b;
}
</style>
