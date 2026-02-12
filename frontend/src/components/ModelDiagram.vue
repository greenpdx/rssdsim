<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import type { LayoutResult, NodeLayout, EdgeLayout } from '@/types'

const props = defineProps<{
  layout: LayoutResult | null
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const zoomLevel = ref(1.0)
const panX = ref(0)
const panY = ref(0)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const selectedNode = ref<NodeLayout | null>(null)

const emit = defineEmits<{
  selectNode: [node: NodeLayout | null]
}>()

function drawDiagram() {
  if (!canvasRef.value) {
    console.error('ModelDiagram: No canvas ref')
    return
  }

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    console.error('ModelDiagram: Could not get 2D context')
    return
  }

  // Get actual container dimensions from the DOM
  const container = canvas.parentElement
  if (!container) {
    console.error('ModelDiagram: No parent container')
    return
  }

  const rect = container.getBoundingClientRect()
  const width = Math.floor(rect.width)
  const height = Math.floor(rect.height)

  if (width < 10 || height < 10) {
    console.warn(`Container too small (${width}x${height}), waiting for proper size`)
    return
  }

  // Set canvas resolution (actual pixels)
  const dpr = window.devicePixelRatio || 1
  canvas.width = width * dpr
  canvas.height = height * dpr

  // Set canvas display size (CSS pixels)
  canvas.style.width = width + 'px'
  canvas.style.height = height + 'px'

  // Scale context to account for device pixel ratio
  ctx.scale(dpr, dpr)

  // Clear canvas with white background
  ctx.fillStyle = '#ffffff'
  ctx.fillRect(0, 0, width, height)

  // Draw border to show canvas bounds
  ctx.strokeStyle = '#e0e0e0'
  ctx.lineWidth = 1
  ctx.strokeRect(0, 0, width, height)

  if (!props.layout) {
    console.log('ModelDiagram: No layout data - showing placeholder')
    ctx.fillStyle = '#666666'
    ctx.font = '16px sans-serif'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText('No model loaded', width / 2, height / 2)
    return
  }

  // Log key info for debugging
  if (props.layout.nodes.length > 0) {
    console.log(`Drawing ${props.layout.nodes.length} nodes at ${Math.round(zoomLevel.value * 100)}% zoom`)
  }

  try {
    // Save context state
    ctx.save()

    // Apply transformations: pan first, then zoom
    ctx.translate(panX.value, panY.value)
    ctx.scale(zoomLevel.value, zoomLevel.value)

    // Draw edges first (behind nodes)
    drawEdges(ctx, props.layout.edges, props.layout.nodes)

    // Draw nodes on top
    drawNodes(ctx, props.layout.nodes)

    // Restore context
    ctx.restore()
  } catch (error) {
    console.error('Error drawing diagram:', error)
    ctx.restore()
  }
}

function drawEdges(ctx: CanvasRenderingContext2D, edges: EdgeLayout[], nodes: NodeLayout[]) {
  ctx.strokeStyle = '#6b7280'
  ctx.lineWidth = 2

  edges.forEach((edge) => {
    const fromNode = nodes.find((n) => n.id === edge.from)
    const toNode = nodes.find((n) => n.id === edge.to)

    if (fromNode && toNode) {
      ctx.beginPath()
      ctx.moveTo(fromNode.x, fromNode.y)
      ctx.lineTo(toNode.x, toNode.y)
      ctx.stroke()

      // Draw arrow
      const angle = Math.atan2(toNode.y - fromNode.y, toNode.x - fromNode.x)
      const arrowSize = 10
      ctx.save()
      ctx.translate(toNode.x, toNode.y)
      ctx.rotate(angle)
      ctx.beginPath()
      ctx.moveTo(0, 0)
      ctx.lineTo(-arrowSize, -arrowSize / 2)
      ctx.lineTo(-arrowSize, arrowSize / 2)
      ctx.closePath()
      ctx.fill()
      ctx.restore()
    }
  })
}

function drawNodes(ctx: CanvasRenderingContext2D, nodes: NodeLayout[]) {
  const colors: Record<string, { fill: string; stroke: string }> = {
    stock: { fill: '#3b82f6', stroke: '#1e40af' },
    flow: { fill: '#f59e0b', stroke: '#d97706' },
    auxiliary: { fill: '#10b981', stroke: '#059669' },
    parameter: { fill: '#8b5cf6', stroke: '#7c3aed' },
  }

  nodes.forEach((node, index) => {
    try {
      const x = node.x - node.width / 2
      const y = node.y - node.height / 2

      const isSelected = selectedNode.value?.id === node.id

      // Get color with fallback for unknown types
      const color = colors[node.type] || { fill: '#9ca3af', stroke: '#6b7280' }

      ctx.fillStyle = color.fill
      ctx.strokeStyle = isSelected ? '#ef4444' : color.stroke
      ctx.lineWidth = isSelected ? 4 : 2

      if (node.type === 'stock') {
        // Rectangle for stocks
        ctx.fillRect(x, y, node.width, node.height)
        ctx.strokeRect(x, y, node.width, node.height)
      } else {
        // Circle for others
        ctx.beginPath()
        ctx.arc(node.x, node.y, node.width / 2, 0, 2 * Math.PI)
        ctx.fill()
        ctx.stroke()
      }

      // Draw label
      ctx.fillStyle = node.type === 'stock' ? '#ffffff' : '#000000'
      ctx.font = '12px sans-serif'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText(node.label || node.id, node.x, node.y)
    } catch (error) {
      console.error(`Error drawing node ${node.id}:`, error)
    }
  })
}

function handleWheel(event: WheelEvent) {
  event.preventDefault()
  const delta = event.deltaY > 0 ? 0.9 : 1.1
  zoomLevel.value = Math.max(0.1, Math.min(5, zoomLevel.value * delta))
  drawDiagram()
}

function handleMouseDown(event: MouseEvent) {
  if (!canvasRef.value || !props.layout) return

  const canvas = canvasRef.value
  const rect = canvas.getBoundingClientRect()
  const mouseX = event.clientX - rect.left
  const mouseY = event.clientY - rect.top

  // Transform mouse coordinates to world coordinates
  // Inverse of: ctx.translate(pan) then ctx.scale(zoom)
  const worldX = (mouseX - panX.value) / zoomLevel.value
  const worldY = (mouseY - panY.value) / zoomLevel.value

  // Check if clicked on a node
  let clickedNode: NodeLayout | null = null
  for (const node of props.layout.nodes) {
    const halfWidth = node.width / 2
    const halfHeight = node.height / 2

    if (
      worldX >= node.x - halfWidth &&
      worldX <= node.x + halfWidth &&
      worldY >= node.y - halfHeight &&
      worldY <= node.y + halfHeight
    ) {
      clickedNode = node
      break
    }
  }

  if (clickedNode) {
    selectedNode.value = clickedNode
    emit('selectNode', clickedNode)
    drawDiagram()
  } else {
    isDragging.value = true
    dragStartX.value = event.clientX - panX.value
    dragStartY.value = event.clientY - panY.value
  }
}

function handleMouseMove(event: MouseEvent) {
  if (!isDragging.value) return
  panX.value = event.clientX - dragStartX.value
  panY.value = event.clientY - dragStartY.value
  drawDiagram()
}

function handleMouseUp() {
  isDragging.value = false
}

function zoomIn() {
  zoomLevel.value = Math.min(5, zoomLevel.value * 1.2)
  drawDiagram()
}

function zoomOut() {
  zoomLevel.value = Math.max(0.1, zoomLevel.value / 1.2)
  drawDiagram()
}

function resetZoom() {
  zoomLevel.value = 1.0
  panX.value = 0
  panY.value = 0
  drawDiagram()
}

// Watch for layout changes
watch(
  () => props.layout,
  (newLayout) => {
    if (newLayout && canvasRef.value) {
      // Auto-fit the diagram to viewport
      const container = canvasRef.value.parentElement
      if (container) {
        const rect = container.getBoundingClientRect()
        const scaleX = (rect.width - 100) / newLayout.width
        const scaleY = (rect.height - 100) / newLayout.height
        const autoScale = Math.min(scaleX, scaleY, 1.0) // Don't zoom in beyond 100%

        zoomLevel.value = autoScale
        panX.value = 50
        panY.value = 50
      }
    }
    // Use nextTick to ensure DOM is updated
    setTimeout(drawDiagram, 10)
  },
  { deep: true }
)

onMounted(() => {
  // Add mouse event listeners
  if (canvasRef.value) {
    canvasRef.value.addEventListener('wheel', handleWheel, { passive: false })
    canvasRef.value.addEventListener('mousedown', handleMouseDown)
    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('mouseup', handleMouseUp)
  }

  // Handle window resize
  const resizeObserver = new ResizeObserver(() => {
    drawDiagram()
  })

  if (canvasRef.value?.parentElement) {
    resizeObserver.observe(canvasRef.value.parentElement)
  }

  // Initial draw attempts
  requestAnimationFrame(() => {
    drawDiagram()
    setTimeout(drawDiagram, 100)
    setTimeout(drawDiagram, 500)
  })
})
</script>

<template>
  <div class="diagram-container">
    <div class="zoom-controls">
      <button @click="zoomIn" title="Zoom In">➕</button>
      <button @click="zoomOut" title="Zoom Out">➖</button>
      <button @click="resetZoom" title="Reset Zoom">🔄</button>
      <span class="zoom-level">{{ Math.round(zoomLevel * 100) }}%</span>
    </div>
    <canvas ref="canvasRef" class="diagram-canvas"></canvas>
  </div>
</template>

<style scoped>
.diagram-container {
  width: 100%;
  height: 100%;
  min-height: 500px;
  background: #fafafa;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

.diagram-canvas {
  flex: 1;
  width: 100%;
  height: 100%;
  display: block;
  cursor: grab;
  background: white;
}

.diagram-canvas:active {
  cursor: grabbing;
}

.zoom-controls {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  gap: 5px;
  z-index: 10;
  background: white;
  padding: 8px;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.zoom-controls button {
  width: 32px;
  height: 32px;
  border: 1px solid #e5e7eb;
  background: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.zoom-controls button:hover {
  background: #f0f9ff;
  border-color: #3b82f6;
}

.zoom-level {
  display: flex;
  align-items: center;
  padding: 0 8px;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  min-width: 50px;
  justify-content: center;
}
</style>
