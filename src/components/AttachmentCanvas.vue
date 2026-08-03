<template>
  <div class="attachment-canvas">
    <div class="canvas-toolbar" :class="{ 'is-readonly': readonly }">
      <div class="toolbar-group" v-if="!readonly">
        <el-tooltip content="自由画笔"><el-button :type="tool === 'pen' ? 'primary' : 'default'" @click="tool = 'pen'"><el-icon><EditPen /></el-icon></el-button></el-tooltip>
        <el-tooltip content="矩形框"><el-button :type="tool === 'rectangle' ? 'primary' : 'default'" @click="tool = 'rectangle'"><el-icon><Crop /></el-icon></el-button></el-tooltip>
        <el-tooltip content="箭头"><el-button :type="tool === 'arrow' ? 'primary' : 'default'" @click="tool = 'arrow'"><el-icon><TopRight /></el-icon></el-button></el-tooltip>
        <el-tooltip content="文字"><el-button :type="tool === 'text' ? 'primary' : 'default'" @click="tool = 'text'"><el-icon><Edit /></el-icon></el-button></el-tooltip>
        <label class="color-control" title="标注颜色"><input v-model="color" type="color" /></label>
        <el-select v-model="strokeWidth" class="stroke-select" aria-label="线条粗细">
          <el-option :value="2" label="细线" /><el-option :value="4" label="中线" /><el-option :value="6" label="粗线" />
        </el-select>
        <el-button :disabled="!canUndo" @click="undo"><el-icon><RefreshLeft /></el-icon>撤销</el-button>
        <el-button :disabled="!pageAnnotations.length" @click="clearPage"><el-icon><Delete /></el-icon>清除本页</el-button>
      </div>
      <div class="toolbar-group viewer-controls">
        <el-button @click="zoom = Math.max(0.5, zoom - 0.1)"><el-icon><ZoomOut /></el-icon></el-button>
        <span class="zoom-value">{{ Math.round(zoom * 100) }}%</span>
        <el-button @click="zoom = Math.min(2.5, zoom + 0.1)"><el-icon><ZoomIn /></el-icon></el-button>
        <template v-if="isPdf">
          <el-divider direction="vertical" />
          <el-button :disabled="pageNumber <= 1" @click="pageNumber--"><el-icon><ArrowLeft /></el-icon></el-button>
          <span class="page-value">第 {{ pageNumber }} / {{ pageCount }} 页</span>
          <el-button :disabled="pageNumber >= pageCount" @click="pageNumber++"><el-icon><ArrowRight /></el-icon></el-button>
        </template>
        <el-switch v-model="showAnnotations" inline-prompt active-text="标注" inactive-text="原图" />
      </div>
    </div>

    <div class="canvas-viewport" :class="{ 'is-drawing': !readonly }">
      <div v-if="loading" class="canvas-state"><el-icon class="is-loading"><Loading /></el-icon><span>正在加载附件</span></div>
      <div v-else-if="error" class="canvas-state is-error"><el-icon><Warning /></el-icon><span>{{ error }}</span></div>
      <div v-show="!error" class="canvas-stage" :style="stageStyle" :aria-busy="loading">
        <canvas ref="contentCanvasRef" class="content-canvas" />
        <canvas
          ref="annotationCanvasRef"
          class="annotation-canvas"
          :class="{ 'is-readonly': readonly || !showAnnotations }"
          @pointerdown="handlePointerDown"
          @pointermove="handlePointerMove"
          @pointerup="handlePointerUp"
          @pointerleave="handlePointerUp"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import type { AttachmentAnnotation, AttachmentContent, AnnotationTool } from '../api'

const props = withDefaults(defineProps<{
  content: AttachmentContent | null
  modelValue: AttachmentAnnotation[]
  readonly?: boolean
}>(), { readonly: false })

const emit = defineEmits<{
  'update:modelValue': [value: AttachmentAnnotation[]]
  change: [value: AttachmentAnnotation[]]
}>()

const contentCanvasRef = ref<HTMLCanvasElement | null>(null)
const annotationCanvasRef = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref('')
const tool = ref<AnnotationTool>('pen')
const color = ref('#ef4444')
const strokeWidth = ref(4)
const zoom = ref(1)
const pageNumber = ref(1)
const pageCount = ref(1)
const showAnnotations = ref(true)
const baseWidth = ref(900)
const baseHeight = ref(600)
let pdfDocument: any = null
let currentDraft: AttachmentAnnotation | null = null

const isPdf = computed(() => props.content?.attachment.mimeType === 'application/pdf')
const pageAnnotations = computed(() => props.modelValue.filter(item => item.page === pageNumber.value))
const canUndo = computed(() => pageAnnotations.value.length > 0)
const stageStyle = computed(() => ({ width: `${baseWidth.value * zoom.value}px`, height: `${baseHeight.value * zoom.value}px` }))

function base64Bytes(value: string) {
  const binary = atob(value)
  return Uint8Array.from(binary, char => char.charCodeAt(0))
}

async function loadContent() {
  error.value = ''
  pdfDocument = null
  pageNumber.value = 1
  pageCount.value = 1
  if (!props.content) return
  loading.value = true
  try {
    if (isPdf.value) {
      const pdfjs = await import('pdfjs-dist')
      const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url')
      pdfjs.GlobalWorkerOptions.workerSrc = worker.default
      pdfDocument = await pdfjs.getDocument({ data: base64Bytes(props.content.data) }).promise
      pageCount.value = pdfDocument.numPages
      await renderPdfPage()
    } else {
      await renderImage()
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : '附件加载失败'
  } finally {
    loading.value = false
  }
}

async function prepareCanvases(width: number, height: number) {
  baseWidth.value = Math.max(1, Math.round(width))
  baseHeight.value = Math.max(1, Math.round(height))
  await nextTick()
  for (const canvas of [contentCanvasRef.value, annotationCanvasRef.value]) {
    if (!canvas) continue
    canvas.width = baseWidth.value
    canvas.height = baseHeight.value
  }
}

async function renderImage() {
  if (!props.content) return
  const image = new Image()
  image.src = `data:${props.content.attachment.mimeType};base64,${props.content.data}`
  await image.decode()
  const maxWidth = 1100
  const scale = Math.min(1, maxWidth / image.naturalWidth)
  await prepareCanvases(image.naturalWidth * scale, image.naturalHeight * scale)
  contentCanvasRef.value?.getContext('2d')?.drawImage(image, 0, 0, baseWidth.value, baseHeight.value)
  drawAnnotations()
}

async function renderPdfPage() {
  if (!pdfDocument) return
  loading.value = true
  try {
    const page = await pdfDocument.getPage(pageNumber.value)
    const initial = page.getViewport({ scale: 1 })
    const scale = Math.min(1.6, 1000 / initial.width)
    const viewport = page.getViewport({ scale })
    await prepareCanvases(viewport.width, viewport.height)
    const canvas = contentCanvasRef.value
    const context = canvas?.getContext('2d')
    if (canvas && context) await page.render({ canvasContext: context, viewport }).promise
    drawAnnotations()
  } finally {
    loading.value = false
  }
}

function normalizedPoint(event: PointerEvent) {
  const rect = annotationCanvasRef.value!.getBoundingClientRect()
  return {
    x: Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width)),
    y: Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height)),
  }
}

async function handlePointerDown(event: PointerEvent) {
  if (props.readonly || !showAnnotations.value || !annotationCanvasRef.value) return
  annotationCanvasRef.value.setPointerCapture(event.pointerId)
  const point = normalizedPoint(event)
  if (tool.value === 'text') {
    try {
      const result = await ElMessageBox.prompt('输入标注文字', '添加文字标注', { inputPlaceholder: '例如：此处尺寸需复核', confirmButtonText: '添加' })
      const annotation: AttachmentAnnotation = {
        id: crypto.randomUUID(), page: pageNumber.value, tool: 'text', color: color.value,
        strokeWidth: strokeWidth.value, x: point.x, y: point.y, text: result.value,
      }
      commitAnnotation(annotation)
    } catch { /* 用户取消 */ }
    return
  }
  currentDraft = {
    id: crypto.randomUUID(), page: pageNumber.value, tool: tool.value, color: color.value,
    strokeWidth: strokeWidth.value, x: point.x, y: point.y, endX: point.x, endY: point.y,
    points: tool.value === 'pen' ? [point] : undefined,
  }
}

function handlePointerMove(event: PointerEvent) {
  if (!currentDraft) return
  const point = normalizedPoint(event)
  currentDraft.endX = point.x
  currentDraft.endY = point.y
  if (currentDraft.tool === 'pen') currentDraft.points?.push(point)
  drawAnnotations(currentDraft)
}

function handlePointerUp(event: PointerEvent) {
  if (!currentDraft) return
  if (annotationCanvasRef.value?.hasPointerCapture(event.pointerId)) annotationCanvasRef.value.releasePointerCapture(event.pointerId)
  commitAnnotation(currentDraft)
  currentDraft = null
}

function commitAnnotation(annotation: AttachmentAnnotation) {
  const next = [...props.modelValue, annotation]
  emit('update:modelValue', next)
  emit('change', next)
  nextTick(() => drawAnnotations())
}

function undo() {
  const index = props.modelValue.map(item => item.page).lastIndexOf(pageNumber.value)
  if (index < 0) return
  const next = [...props.modelValue]
  next.splice(index, 1)
  emit('update:modelValue', next)
  emit('change', next)
}

function clearPage() {
  const next = props.modelValue.filter(item => item.page !== pageNumber.value)
  emit('update:modelValue', next)
  emit('change', next)
}

function drawArrow(context: CanvasRenderingContext2D, startX: number, startY: number, endX: number, endY: number) {
  const angle = Math.atan2(endY - startY, endX - startX)
  const size = 12
  context.beginPath(); context.moveTo(startX, startY); context.lineTo(endX, endY); context.stroke()
  context.beginPath(); context.moveTo(endX, endY)
  context.lineTo(endX - size * Math.cos(angle - Math.PI / 6), endY - size * Math.sin(angle - Math.PI / 6))
  context.moveTo(endX, endY)
  context.lineTo(endX - size * Math.cos(angle + Math.PI / 6), endY - size * Math.sin(angle + Math.PI / 6))
  context.stroke()
}

function drawOne(context: CanvasRenderingContext2D, item: AttachmentAnnotation) {
  const width = baseWidth.value, height = baseHeight.value
  const x = item.x * width, y = item.y * height
  const endX = (item.endX ?? item.x) * width, endY = (item.endY ?? item.y) * height
  context.strokeStyle = item.color; context.fillStyle = item.color; context.lineWidth = item.strokeWidth
  context.lineCap = 'round'; context.lineJoin = 'round'
  if (item.tool === 'pen' && item.points?.length) {
    context.beginPath()
    item.points.forEach((point, index) => index ? context.lineTo(point.x * width, point.y * height) : context.moveTo(point.x * width, point.y * height))
    context.stroke()
  } else if (item.tool === 'rectangle') {
    context.strokeRect(x, y, endX - x, endY - y)
  } else if (item.tool === 'arrow') {
    drawArrow(context, x, y, endX, endY)
  } else if (item.tool === 'text' && item.text) {
    context.font = `600 ${Math.max(16, item.strokeWidth * 5)}px "Microsoft YaHei", sans-serif`
    context.fillText(item.text, x, y)
  }
}

function drawAnnotations(draft?: AttachmentAnnotation) {
  const canvas = annotationCanvasRef.value
  const context = canvas?.getContext('2d')
  if (!canvas || !context) return
  context.clearRect(0, 0, canvas.width, canvas.height)
  if (!showAnnotations.value) return
  pageAnnotations.value.forEach(item => drawOne(context, item))
  if (draft) drawOne(context, draft)
}

watch(() => props.content?.attachment.id, loadContent, { immediate: true })
watch(pageNumber, () => { if (isPdf.value) renderPdfPage() })
watch(() => props.modelValue, () => drawAnnotations(), { deep: true })
watch(showAnnotations, () => drawAnnotations())
watch(zoom, () => nextTick(() => drawAnnotations()))
async function exportRenderedPng() {
  const contentCanvas = contentCanvasRef.value
  const annotationCanvas = annotationCanvasRef.value
  if (!contentCanvas || !annotationCanvas || loading.value || error.value) {
    throw new Error('附件尚未完成渲染')
  }
  const output = document.createElement('canvas')
  output.width = contentCanvas.width
  output.height = contentCanvas.height
  const context = output.getContext('2d')
  if (!context) throw new Error('无法创建导出画布')
  context.fillStyle = '#ffffff'
  context.fillRect(0, 0, output.width, output.height)
  context.drawImage(contentCanvas, 0, 0)
  if (showAnnotations.value) context.drawImage(annotationCanvas, 0, 0)
  const blob = await new Promise<Blob | null>(resolve => output.toBlob(resolve, 'image/png'))
  if (!blob) throw new Error('生成标注副本失败')
  return {
    bytes: new Uint8Array(await blob.arrayBuffer()),
    pageNumber: pageNumber.value,
    isPdf: isPdf.value,
  }
}

defineExpose({ exportRenderedPng })
onBeforeUnmount(() => { pdfDocument?.destroy?.() })
</script>

<style scoped>
.attachment-canvas { height: 100%; min-height: 0; display: flex; flex-direction: column; background: var(--bg); }
.canvas-toolbar { min-height: 54px; padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; border-bottom: 1px solid var(--border); background: var(--card-bg); }
.toolbar-group { display: flex; align-items: center; gap: 6px; }
.viewer-controls { margin-left: auto; }
.color-control { width: 32px; height: 32px; padding: 3px; display: grid; place-items: center; border: 1px solid var(--border); border-radius: 7px; background: var(--card-bg); }
.color-control input { width: 23px; height: 23px; padding: 0; border: 0; background: transparent; cursor: pointer; }
.stroke-select { width: 82px; }
.zoom-value, .page-value { min-width: 54px; color: var(--text-secondary); font-size: 12px; text-align: center; }
.page-value { min-width: 92px; }
.canvas-viewport { position: relative; flex: 1; min-height: 0; padding: 24px; overflow: auto; display: flex; align-items: flex-start; justify-content: center; background: color-mix(in srgb, var(--bg) 82%, #64748b); }
.canvas-stage { position: relative; flex: 0 0 auto; background: white; box-shadow: 0 18px 48px rgba(15, 23, 42, 0.22); transform-origin: top center; }
.canvas-stage[aria-busy="true"] { visibility: hidden; }
.content-canvas, .annotation-canvas { position: absolute; inset: 0; width: 100%; height: 100%; }
.annotation-canvas { touch-action: none; cursor: crosshair; }
.annotation-canvas.is-readonly { pointer-events: none; cursor: default; }
.canvas-state { position: absolute; inset: 0; z-index: 2; min-height: 320px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--text-muted); }
.canvas-state .el-icon { font-size: 28px; }
.canvas-state.is-error { color: var(--el-color-danger); }
@media (max-width: 900px) { .canvas-toolbar { align-items: flex-start; } .viewer-controls { margin-left: 0; } .canvas-viewport { padding: 12px; justify-content: flex-start; } }
</style>
