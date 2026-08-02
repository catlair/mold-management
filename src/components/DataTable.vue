<template>
  <div ref="wrapRef" class="dt-wrap">
    <vxe-table
      ref="tableRef"
      :data="data"
      :loading="loading"
      border
      round
      stripe
      align="center"
      :fit="false"
      :row-config="{ keyField: 'id' }"
      show-overflow
      class="dt-table"
    >
      <slot />
    </vxe-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps<{
  data: any[]
  loading?: boolean
}>()

const wrapRef = ref<HTMLDivElement | null>(null)
const tableRef = ref<any>(null)

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
  scrollbar-width: thin;
  scrollbar-color: #b0b4bc #f0f2f5;
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
/* 表头吸顶 */
.dt-wrap :deep(.vxe-table--header-wrapper) {
  position: sticky;
  top: 0;
  z-index: 5;
  background: #fafbfc;
}
.dt-wrap :deep(.vxe-header--column) {
  background: #fafbfc;
  color: #303133;
  font-weight: 600;
  font-size: 13px;
}
/* Webkit 滚动条样式（Chromium / WebView2） */
.dt-wrap::-webkit-scrollbar {
  width: 12px;
  height: 12px;
}
.dt-wrap::-webkit-scrollbar-track {
  background: #f0f2f5;
  border-radius: 6px;
}
.dt-wrap::-webkit-scrollbar-thumb {
  background: #b0b4bc;
  border-radius: 6px;
  border: 2px solid #f0f2f5;
}
.dt-wrap::-webkit-scrollbar-thumb:hover {
  background: #909399;
}
.dt-wrap::-webkit-scrollbar-thumb:active {
  background: #606266;
}
</style>