<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><List /></el-icon>
          <span>操作日志</span>
          <div class="header-right">
            <el-button :loading="loading" @click="loadLogs">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button type="danger" plain :disabled="!total" @click="handleClear">
              <el-icon><Delete /></el-icon>
              清空日志
            </el-button>
          </div>
        </div>
      </template>

      <div class="log-toolbar">
        <el-select v-model="filters.operation" placeholder="操作类型" clearable style="width: 140px" @change="applyFilters">
          <el-option label="新增" value="add" />
          <el-option label="修改" value="update" />
          <el-option label="删除" value="delete" />
          <el-option label="导入" value="import" />
        </el-select>
        <el-select v-model="filters.tableName" placeholder="对象" clearable filterable style="width: 200px" @change="applyFilters">
          <el-option v-for="name in tableOptions" :key="name" :label="name" :value="name" />
        </el-select>
        <el-input
          v-model="filters.keyword"
          placeholder="搜索摘要或记录 ID..."
          clearable
          style="width: 260px"
          @input="applyFilters"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
        <span class="log-count">共 {{ filtered.length }} 条{{ total ? `（库内共 ${total} 条）` : '' }}</span>
      </div>

      <div class="log-table-wrap">
        <el-table v-loading="loading" :data="paginatedRows" stripe height="100%">
          <el-table-column label="时间" width="170">
            <template #default="{ row }">{{ formatTime(row.ts) }}</template>
          </el-table-column>
          <el-table-column label="类型" width="90">
            <template #default="{ row }">
              <el-tag :type="operationMeta(row.operation).type" size="small">
                {{ operationMeta(row.operation).label }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="tableName" label="对象" width="170" show-overflow-tooltip />
          <el-table-column prop="summary" label="操作内容" min-width="320" show-overflow-tooltip />
          <el-table-column prop="recordId" label="记录 ID" width="130" show-overflow-tooltip />
        </el-table>
      </div>

      <div class="log-footer">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[50, 100, 200]"
          layout="total, sizes, prev, pager, next, jumper"
          :total="filtered.length"
          @current-change="onPageChange"
          @size-change="onPageChange"
        />
      </div>

      <div class="log-note">
        <el-icon><InfoFilled /></el-icon>
        <span>操作日志记录新增、修改、删除与导入操作，保留 365 天（最多 5 万条），随数据备份与同步一起保存。清空操作不可恢复。</span>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { operationLogApi, type OperationLogEntry } from '../api'

const loading = ref(false)
const allLogs = ref<OperationLogEntry[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(100)

const filters = reactive({
  operation: '' as '' | OperationLogEntry['operation'],
  tableName: '',
  keyword: '',
})

const tableOptions = computed(() => [...new Set(allLogs.value.map(log => log.tableName).filter(Boolean))].sort())

const filtered = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase()
  return allLogs.value.filter((log) => {
    if (filters.operation && log.operation !== filters.operation) return false
    if (filters.tableName && log.tableName !== filters.tableName) return false
    if (keyword && !log.summary.toLowerCase().includes(keyword) && !log.recordId.toLowerCase().includes(keyword)) return false
    return true
  })
})

const paginatedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filtered.value.slice(start, start + pageSize.value)
})

function formatTime(ts: number): string {
  const date = new Date(ts * 1000)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}

function operationMeta(operation: OperationLogEntry['operation']): { label: string; type: 'success' | 'warning' | 'danger' | 'primary' } {
  switch (operation) {
    case 'add': return { label: '新增', type: 'success' }
    case 'update': return { label: '修改', type: 'warning' }
    case 'delete': return { label: '删除', type: 'danger' }
    case 'import': return { label: '导入', type: 'primary' }
    default: return { label: operation, type: 'primary' }
  }
}

function onPageChange() {
  // 分页状态已由 v-model 维护，无需额外处理
}

function applyFilters() {
  currentPage.value = 1
}

async function loadLogs() {
  loading.value = true
  try {
    const result = await operationLogApi.get(500, 0)
    allLogs.value = result.items
    total.value = result.total
  } catch (error) {
    ElMessage.error('加载操作日志失败')
    console.error('加载操作日志失败', error)
  } finally {
    loading.value = false
  }
}

async function handleClear() {
  try {
    await ElMessageBox.confirm(
      '确定清空全部操作日志？此操作不可恢复。',
      '清空操作日志',
      { type: 'warning', confirmButtonText: '清空', cancelButtonText: '取消' },
    )
  } catch {
    return
  }
  try {
    await operationLogApi.clear()
    ElMessage.success('操作日志已清空')
    loadLogs()
  } catch (error) {
    ElMessage.error('清空操作日志失败')
    console.error('清空操作日志失败', error)
  }
}

onMounted(loadLogs)
</script>

<style scoped>
.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-right {
  margin-left: auto;
  display: flex;
  gap: 10px;
}

.log-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.log-count {
  color: var(--text-muted);
  font-size: 12px;
}

.log-table-wrap {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.log-footer {
  flex-shrink: 0;
  display: flex;
  justify-content: flex-end;
  padding-top: 12px;
}

.log-note {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 12px;
}
</style>
