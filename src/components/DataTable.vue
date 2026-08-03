<template>
  <div
    ref="wrapRef"
    class="dt-wrap"
    role="region"
    aria-label="数据表格，可使用方向键滚动"
    tabindex="0"
  >
    <vxe-table
      ref="tableRef"
      :data="data"
      :loading="loading"
      border
      round
      stripe
      align="center"
      :fit="false"
      :id="tableId"
      :row-config="{ keyField: 'id', isHover: true }"
      :column-config="{ resizable: true }"
      :custom-config="customConfig"
      show-header-overflow="tooltip"
      @column-resizable-change="handleColumnResizableChange"
      :empty-text="loading ? '正在加载数据' : '暂无数据'"
      class="dt-table"
    >
      <slot />
      <template #empty>
        <div class="dt-empty" aria-live="polite">
          <el-icon class="dt-empty-icon"><DocumentRemove /></el-icon>
          <span>{{ loading ? '正在加载数据' : '暂无数据' }}</span>
        </div>
      </template>
    </vxe-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, provide, toRef } from 'vue'
import { DocumentRemove } from '@element-plus/icons-vue'
import {
  createVxeCustomConfig,
  resolveColumnResize,
  useTablePreferences,
  type ColumnResizeEventParams,
} from '../composables/useTablePreferences'

const props = withDefaults(defineProps<{
  tableId?: string
  data: any[]
  loading?: boolean
}>(), {
  tableId: '',
})

provide('tablePreferenceContext', {
  tableId: toRef(props, 'tableId'),
  data: toRef(props, 'data'),
})

const wrapRef = ref<HTMLDivElement | null>(null)
const tableRef = ref<any>(null)
const customConfig = createVxeCustomConfig()
const { setColumnPreference } = useTablePreferences()

function handleColumnResizableChange(params: ColumnResizeEventParams) {
  if (!props.tableId) return
  const resized = resolveColumnResize(params)
  if (!resized) return

  setColumnPreference(props.tableId, resized.columnId, { width: resized.width })
  nextTick(() => {
    tableRef.value?.recalculate?.()
    window.dispatchEvent(new Event('resize'))
  })
}

defineExpose({ tableRef })

/**
 * 桌面应用兼容性修复（WebView2 / Edge 内核）：
 * - vxe-table 的 :height 在 Chrome 可触发内部滚动，WebView2 行为不一致（撑开但不滚）
 * - 方案：vxe-table 不传 height（自然展开），由父容器 dt-wrap 承担纵横向滚动
 * - dt-wrap 高度由 flex 链（app-main → page-container → el-card → el-card__body → [el-tabs→content→pane] → dt-wrap）自适应
 * - measure() 仅做兜底：flex 链异常时用视口估算高度，避免表格撑爆页面被裁
 */
function measure() {
  const wrap = wrapRef.value
  if (!wrap) return
  // flex 链正常时 dt-wrap 已有合理高度，不干预
  if (wrap.clientHeight > 60) return
  // 兜底：祖先链断裂或测量异常时基于视口估算
  const h = Math.max(400, (typeof window !== 'undefined' ? window.innerHeight : 800) - 220)
  if (parseInt(wrap.style.height || '0') !== h) {
    wrap.style.height = h + 'px'
    nextTick(() => tableRef.value?.recalculate?.())
  }
}

let observer: ResizeObserver | null = null
onMounted(() => {
  requestAnimationFrame(() => measure())
  observer = new ResizeObserver(() => measure())
  if (wrapRef.value) observer.observe(wrapRef.value)
})
onUnmounted(() => { observer?.disconnect() })
</script>

<style scoped>
.dt-wrap {
  overflow: auto;
  min-height: 0;
  width: 100%;
  flex: 1;
  /* 桌面应用兼容：确保滚动条可见（Chrome/Edge/WebView2） */
  overscroll-behavior: contain;
  scrollbar-gutter: stable both-edges;
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  border-radius: 9px;
  outline: none;
  transition: box-shadow 0.18s ease, background-color 0.18s ease;
}

.dt-wrap:focus-visible {
  box-shadow: 0 0 0 2px var(--focus-ring);
}
.dt-table {
  width: max-content;
  min-width: 100%;
}
/* 横向溢出由 dt-wrap 接管（vxe-table 内部不滚） */
.dt-wrap :deep(.vxe-table) {
  width: max-content;
  min-width: 100%;
}
.dt-wrap :deep(.vxe-table .vxe-table--body-wrapper) {
  overflow-x: visible !important;
}
.dt-wrap :deep(.vxe-table .el-table__body-wrapper) {
  overflow-x: visible !important;
}
/* 表头由全局滚动同步器固定，避免 vxe 包装层导致 CSS sticky 在 WebView2 失效 */
.dt-wrap :deep(.vxe-table--header-wrapper) {
  background: var(--surface-muted);
}
.dt-wrap :deep(.vxe-header--column) {
  background: var(--surface-muted);
  color: var(--text-primary);
  font-weight: 600;
  font-size: 13px;
}
/* Webkit 滚动条样式（Chromium / WebView2） */
.dt-wrap::-webkit-scrollbar {
  width: 12px;
  height: 12px;
}
.dt-wrap::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
  border-radius: 6px;
}
.dt-wrap::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 6px;
  border: 2px solid var(--scrollbar-track);
}
.dt-wrap::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}
.dt-wrap::-webkit-scrollbar-thumb:active {
  background: var(--text-secondary);
}

.dt-empty {
  min-height: 180px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-muted);
  font-size: 13px;
}

.dt-empty-icon {
  font-size: 30px;
  opacity: 0.72;
}

.dt-wrap :deep(.vxe-table--body tr) {
  transition: background-color 0.14s ease;
}

/* 规格字段优先完整显示：允许正文自动换行并由内容撑高行高 */
.dt-wrap :deep(.vxe-body--column .vxe-cell) {
  height: auto;
  min-height: 46px;
  padding-top: 10px;
  padding-bottom: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1.45;
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.dt-wrap :deep(.vxe-body--column .vxe-cell--wrapper) {
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.dt-wrap :deep(.vxe-body--row) {
  height: auto !important;
}

@media (prefers-reduced-motion: reduce) {
  .dt-wrap,
  .dt-wrap :deep(.vxe-table--body tr) {
    transition: none;
  }
}
</style>