<template>
  <div class="page-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <div class="title-block">
            <el-icon><Operation /></el-icon>
            <div>
              <strong>页面管理</strong>
              <span>配置各页面表格的显示列、排序和筛选；在列表表头拖动列边界可调整宽度。</span>
            </div>
          </div>
          <el-button @click="resetAll">恢复全部默认</el-button>
        </div>
      </template>

      <div class="management-layout">
        <aside class="page-list" aria-label="页面列表">
          <button
            v-for="page in pageTableCatalog"
            :key="page.id"
            type="button"
            class="page-list-item"
            :class="{ active: page.id === activePageId }"
            @click="activePageId = page.id"
          >
            <span>{{ page.label }}</span>
            <small>{{ page.tables.length }} 个表格</small>
          </button>
        </aside>

        <main v-if="activePage" class="table-settings">
          <div class="page-summary">
            <div>
              <h2>{{ activePage.label }}</h2>
              <p>修改后自动保存，并在对应页面即时生效。</p>
            </div>
            <el-button type="primary" plain @click="openPage">打开页面</el-button>
          </div>

          <el-collapse v-model="activeTables" class="table-collapse">
            <el-collapse-item v-for="table in activePage.tables" :key="table.id" :name="table.id">
              <template #title>
                <div class="table-title">
                  <span>{{ table.label }}</span>
                  <el-tag size="small" effect="plain">{{ visibleCount(table.id) }}/{{ table.columns.length }} 列显示</el-tag>
                </div>
              </template>

              <div class="table-toolbar">
                <span>列配置 <small>已拖动的列宽会自动保存</small></span>
                <el-button link type="primary" @click.stop="resetTable(table.id)">恢复默认</el-button>
              </div>

              <el-table :data="table.columns" border stripe class="column-table">
                <el-table-column prop="label" label="列名" min-width="180">
                  <template #default="{ row }">
                    <div class="column-name">
                  <strong>{{ row.label }}</strong>
                  <small>{{ row.id }}<template v-if="columnState(table.id, row.id).width"> · {{ columnState(table.id, row.id).width }}px</template></small>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="显示" width="120" align="center">
                  <template #default="{ row }">
                    <el-switch
                      :model-value="columnState(table.id, row.id).visible"
                      @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'visible', Boolean(value))"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="排序" width="120" align="center">
                  <template #default="{ row }">
                    <el-tooltip :disabled="row.canSort !== false" content="该列不支持排序" placement="top">
                      <span>
                        <el-switch
                          :model-value="columnState(table.id, row.id).sortable"
                          :disabled="row.canSort === false"
                          @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'sortable', Boolean(value))"
                        />
                      </span>
                    </el-tooltip>
                  </template>
                </el-table-column>
                <el-table-column label="筛选" width="120" align="center">
                  <template #default="{ row }">
                    <el-tooltip :disabled="row.canFilter !== false" content="该列不支持筛选" placement="top">
                      <span>
                        <el-switch
                          :model-value="columnState(table.id, row.id).filterable"
                          :disabled="row.canFilter === false"
                          @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'filterable', Boolean(value))"
                        />
                      </span>
                    </el-tooltip>
                  </template>
                </el-table-column>
              </el-table>
            </el-collapse-item>
          </el-collapse>
        </main>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Operation } from '@element-plus/icons-vue'
import { pageTableCatalog } from '../config/tableCatalog'
import { useTablePreferences, type ColumnPreference } from '../composables/useTablePreferences'
import { isUserCancellation, showDetailedError } from '../utils/errorFeedback'

const router = useRouter()
const activePageId = ref(pageTableCatalog[0]?.id || '')
const activeTables = ref<string[]>([])
const activePage = computed(() => pageTableCatalog.find(page => page.id === activePageId.value))
const { getColumnPreference, setColumnPreference, resetTablePreference, resetAllPreferences } = useTablePreferences()

watch(activePage, page => {
  activeTables.value = page?.tables.map(table => table.id) || []
}, { immediate: true })

function columnState(tableId: string, columnId: string) {
  return getColumnPreference(tableId, columnId)
}

function updateColumn(tableId: string, columnId: string, key: keyof ColumnPreference, value: boolean) {
  setColumnPreference(tableId, columnId, { [key]: value })
}

function visibleCount(tableId: string) {
  const table = activePage.value?.tables.find(item => item.id === tableId)
  return table?.columns.filter(column => columnState(tableId, column.id).visible).length || 0
}

function resetTable(tableId: string) {
  resetTablePreference(tableId)
  ElMessage.success('已恢复该表格默认配置')
}

async function resetAll() {
  try {
    await ElMessageBox.confirm('确定恢复所有表格的默认列配置？', '恢复默认', { type: 'warning' })
    resetAllPreferences()
    ElMessage.success('已恢复全部默认配置')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('恢复全部表格默认配置', error)
  }
}

function openPage() {
  if (activePage.value) router.push(activePage.value.route)
}
</script>

<style scoped>
.page-management {
  height: 100%;
  min-height: 0;
}

.management-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.management-card :deep(.el-card__body) {
  flex: 1;
  min-height: 0;
  padding: 0;
}

.card-header,
.title-block,
.page-summary,
.table-title,
.table-toolbar,
.column-name {
  display: flex;
  align-items: center;
}

.card-header {
  justify-content: space-between;
  gap: 20px;
}

.title-block {
  gap: 12px;
}

.title-block > .el-icon {
  color: var(--el-color-primary);
  font-size: 22px;
}

.title-block > div,
.column-name {
  flex-direction: column;
  align-items: flex-start;
}

.title-block strong {
  color: var(--text-primary);
  font-size: 16px;
}

.title-block span,
.page-summary p,
.column-name small {
  color: var(--text-secondary);
  font-size: 12px;
}

.management-layout {
  height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
}

.page-list {
  min-height: 0;
  padding: 16px 12px;
  overflow-y: auto;
  background: var(--surface-muted);
  border-right: 1px solid var(--border);
}

.page-list-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 6px;
  padding: 11px 12px;
  color: var(--text-secondary);
  font: inherit;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
}

.page-list-item:hover {
  background: var(--surface-hover);
}

.page-list-item.active {
  color: var(--el-color-primary);
  font-weight: 600;
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-7);
}

.page-list-item small {
  color: var(--text-muted);
  white-space: nowrap;
}

.table-settings {
  min-width: 0;
  min-height: 0;
  padding: 20px 24px 32px;
  overflow: auto;
}

.page-summary {
  justify-content: space-between;
  margin-bottom: 18px;
}

.page-summary h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 20px;
}

.page-summary p {
  margin: 5px 0 0;
}

.table-collapse {
  border-top: 0;
}

.table-collapse :deep(.el-collapse-item) {
  margin-bottom: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.table-collapse :deep(.el-collapse-item__header) {
  height: 52px;
  padding: 0 16px;
  background: var(--surface-muted);
  border-bottom: 0;
}

.table-collapse :deep(.el-collapse-item__wrap) {
  border-bottom: 0;
}

.table-collapse :deep(.el-collapse-item__content) {
  padding: 14px 16px 16px;
}

.table-title,
.table-toolbar {
  width: 100%;
  justify-content: space-between;
  padding-right: 8px;
}

.table-toolbar {
  margin-bottom: 10px;
  color: var(--text-secondary);
  font-size: 13px;
}

.table-toolbar small {
  margin-left: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

.column-table {
  --el-table-border-color: var(--border);
  --el-table-bg-color: var(--card-bg);
  --el-table-tr-bg-color: var(--card-bg);
  --el-table-header-bg-color: var(--surface-muted);
  --el-table-row-hover-bg-color: var(--surface-hover);
  --el-table-current-row-bg-color: var(--surface-hover);
  --el-table-header-text-color: var(--text-primary);
  --el-table-text-color: var(--text-secondary);
  --el-fill-color-lighter: var(--table-stripe-bg);
  overflow: hidden;
  border-color: var(--border);
  background: var(--card-bg);
}

.column-table :deep(.el-table__inner-wrapper::before) {
  background-color: var(--border);
}

.column-table :deep(.el-switch) {
  --el-switch-on-color: var(--el-color-primary);
  --el-switch-off-color: var(--switch-off-bg);
}

.column-table :deep(.el-switch.is-disabled) {
  --el-switch-on-color: color-mix(in srgb, var(--el-color-primary) 46%, var(--card-bg));
  --el-switch-off-color: var(--switch-disabled-bg);
  opacity: 0.72;
}

.column-table :deep(.el-switch__core) {
  border-color: color-mix(in srgb, currentColor 12%, var(--border));
  transition: background-color 160ms ease, border-color 160ms ease;
}

.column-table :deep(.el-switch:not(.is-disabled):focus-visible .el-switch__core) {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.column-name {
  gap: 2px;
}

.column-name strong {
  color: var(--text-primary);
  font-weight: 500;
}

@media (max-width: 900px) {
  .management-layout {
    grid-template-columns: 170px minmax(0, 1fr);
  }

  .table-settings {
    padding: 16px;
  }
}
</style>
