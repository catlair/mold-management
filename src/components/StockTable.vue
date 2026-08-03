<template>
  <div
    class="stock-table-shell"
    role="region"
    :aria-label="`${nameLabel}库存汇总表格`"
    tabindex="0"
  >
    <ConfigurableVxeTable
      :table-id="tableId"
      :data="data"
      :loading="loading"
      :fit="true"
      border
      round
      stripe
      show-header-overflow="tooltip"
      class="stock-table"
    >
      <ConfigurableTable field="name" :title="nameLabel" width="44%" min-width="260" sortable />
      <ConfigurableTable field="currentStock" title="当前库存" width="18%" min-width="140" sortable />
      <ConfigurableTable field="safetyStock" title="安全库存" width="18%" min-width="140" sortable />
      <ConfigurableTable field="status" title="库存状态" width="20%" min-width="160" sortable>
        <template #default="{ row }">
          <el-tag :type="row.status === '需订购' ? 'danger' : 'success'" effect="dark" round>
            {{ row.status }}
          </el-tag>
        </template>
      </ConfigurableTable>
      <template #empty>
        <el-empty description="暂无库存数据" />
      </template>
    </ConfigurableVxeTable>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  tableId: string
  data: any[]
  nameLabel: string
  loading?: boolean
}>()
</script>

<style scoped>
.stock-table-shell {
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding: 14px;
  overflow: auto;
  background: var(--surface-muted);
  border: 1px solid var(--border);
  border-radius: 14px;
  scrollbar-gutter: stable;
  overscroll-behavior: contain;
  transition:
    border-color 180ms ease,
    background-color 180ms ease;
}

.stock-table-shell:focus-visible {
  outline: 2px solid var(--el-color-primary);
  outline-offset: 2px;
}

.stock-table {
  width: 100%;
  min-width: 760px;
  overflow: hidden;
  background: var(--card-bg);
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.05);
}

.stock-table-shell :deep(.vxe-header--column) {
  height: 50px;
  font-weight: 600;
}

.stock-table-shell :deep(.vxe-body--column .vxe-cell) {
  min-height: 48px;
  padding: 10px 16px;
  line-height: 1.45;
  white-space: normal;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.stock-table-shell :deep(.vxe-body--column:first-child .vxe-cell),
.stock-table-shell :deep(.vxe-header--column:first-child .vxe-cell) {
  justify-content: flex-start;
  text-align: left;
}

:global(.dark) .stock-table {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
}

@media (max-width: 900px) {
  .stock-table-shell {
    padding: 10px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .stock-table-shell {
    transition: none;
  }
}
</style>