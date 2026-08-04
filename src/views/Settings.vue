<template>
  <div class="settings-page">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <el-icon><Upload /></el-icon>
          <span>数据导入导出</span>
        </div>
      </template>
      <div class="action-group">
        <div class="action-group-title">完整数据包（推荐用于迁移和完整归档）</div>
        <div class="action-buttons">
          <el-button type="success" @click="handleExportPackage" :loading="packageExporting">
            <el-icon><Download /></el-icon>
            导出完整数据包
          </el-button>
          <el-button type="danger" plain @click="handleImportPackage" :loading="packageImporting">
            <el-icon><Upload /></el-icon>
            导入完整数据包
          </el-button>
        </div>
        <div class="action-desc">
          <p>完整数据包包含数据库、全部附件和版本清单，适合换电脑、完整迁移和离线归档。</p>
        </div>
      </div>
      <div class="action-group secondary-action-group">
        <div class="action-group-title">Excel 导出（按业务分组，每组一个文件）</div>
        <div class="action-buttons">
          <el-button
            v-for="group in exportGroups"
            :key="group.id"
            size="small"
            type="primary"
            plain
            class="export-group-button"
            :loading="exportingGroup === group.id"
            @click="handleExportGroup(group.id)"
          >
            <el-icon><Download /></el-icon>
            {{ group.label }}
          </el-button>
        </div>
        <div class="action-desc">
          <p>每组生成一个独立 Excel（如“冲头.xlsx”包含冲头信息、入库、领用、关联与库存汇总），不再把所有表放进一个文件。</p>
        </div>
      </div>
      <div class="action-group secondary-action-group">
        <div class="action-group-title">Excel 导入（可勾选要导入的工作表）</div>
        <div class="action-buttons">
          <el-button type="warning" @click="handleImport" :loading="importing">
            <el-icon><Upload /></el-icon>
            选择 Excel 并导入
          </el-button>
        </div>
        <div class="action-desc">
          <p>导入前先列出 Excel 中的业务表，勾选后整表替换对应数据；导入前自动创建联合备份。</p>
        </div>
      </div>
    </el-card>

    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <el-icon><Folder /></el-icon>
          <span>数据文件配置</span>
        </div>
      </template>
      <div class="path-config">
        <div class="current-path">
          <span class="label">当前数据文件：</span>
          <el-tag type="info" class="path-tag" :title="dataPath || '加载中...'">
            <span class="path-text">{{ dataPath || '加载中...' }}</span>
          </el-tag>
        </div>
        <div class="path-actions">
          <el-button type="primary" @click="handleSelectPath">
            <el-icon><FolderOpened /></el-icon>
            选择文件
          </el-button>
        </div>
      </div>
      <div class="action-desc">
        <p>选择一个 .db 数据库文件作为数据存储位置</p>
        <p>修改后需要重启应用才能生效</p>
      </div>
    </el-card>

    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <el-icon><Clock /></el-icon>
          <span>自动备份配置</span>
        </div>
      </template>
      <div class="backup-config">
        <el-form label-width="120px">
          <el-form-item label="备份目录">
            <div class="backup-path-row">
              <el-tag
                type="info"
                class="path-tag"
                :title="backupConfig.effectiveBackupDir || '加载中...'"
              >
                <span class="path-text">{{ backupConfig.effectiveBackupDir || '加载中...' }}</span>
              </el-tag>
              <el-button size="small" @click="handleSelectBackupDir">更改</el-button>
              <el-button size="small" type="info" @click="resetBackupDir">恢复默认</el-button>
            </div>
          </el-form-item>
          <el-form-item label="备份策略">
            <el-tag type="success">启动时自动备份</el-tag>
            <el-tag type="success" class="backup-policy-tag">退出时自动备份</el-tag>
          </el-form-item>
        </el-form>
      </div>
      <div class="action-desc">
        <p>每次备份生成一个单文件 ZIP（包含 Excel 与全部附件），备份记录按保留份数自动清理。</p>
      </div>
      <div class="action-buttons backup-action">
        <el-button type="primary" @click="handleBackupNow" :loading="backingUp">
          <el-icon><CopyDocument /></el-icon>
          立即备份
        </el-button>
      </div>
    </el-card>

    <el-card class="settings-card backup-records-card">
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>备份记录</span>
          <el-button size="small" class="refresh-button" @click="loadBackups">刷新</el-button>
        </div>
      </template>
      <div class="backup-table-scroll">
        <ConfigurableVxeTable table-id="settings.backups" :data="backups" :fit="true" class="backup-table" max-height="400">
          <ConfigurableTable field="backup_time" title="备份时间" width="24%" min-width="140" show-overflow="tooltip" />
          <ConfigurableTable field="backup_reason" title="备份原因" width="18%" min-width="100" show-overflow="tooltip" />
          <ConfigurableTable field="backup_md5" title="MD5" width="34%" min-width="200" show-overflow="tooltip" />
          <ConfigurableTable title="锁定" width="10%" min-width="64" align="center">
            <template #default="{ row, rowIndex }">
              <el-button size="small" :type="row.locked ? 'warning' : 'info'" link @click="handleToggleLock(rowIndex)">
                <el-icon><Lock v-if="row.locked" /><Unlock v-else /></el-icon>
              </el-button>
            </template>
          </ConfigurableTable>
          <ConfigurableTable title="操作" width="14%" min-width="96" class-name="operation-column backup-operation-column" header-class-name="operation-column backup-operation-column">
            <template #default="{ row }">
              <el-button size="small" type="warning" @click="handleRestore(row)">恢复</el-button>
            </template>
          </ConfigurableTable>
        </ConfigurableVxeTable>
      </div>
      <div v-if="backups.length === 0" class="backup-empty">
        暂无备份记录
      </div>
    </el-card>

    <el-dialog v-model="importDialogVisible" title="选择要导入的工作表" width="520px" :close-on-click-modal="false">
      <div class="import-sheet-tip">该 Excel 中包含以下业务表，勾选后将整表替换对应数据（库存汇总为系统计算表，默认不导入）：</div>
      <el-checkbox-group v-model="selectedSheets" class="import-sheet-group">
        <el-checkbox v-for="sheet in availableSheets" :key="sheet.name" :value="sheet" class="import-sheet-item">
          <span class="import-sheet-name">
            {{ sheet.matchedByHeader ? `${sheet.name}（识别为${sheet.table}）` : sheet.table }}
          </span>
          <span class="import-sheet-meta">
            {{ sheet.rowCount }} 行
            <el-tag v-if="sheet.systemCalculated" size="small" type="info" class="import-sheet-tag">系统计算表</el-tag>
          </span>
        </el-checkbox>
      </el-checkbox-group>
      <template #footer>
        <el-button @click="importDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="importing" @click="confirmImport">确定导入</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { save, open } from '@tauri-apps/plugin-dialog'
import { dataApi, settingsApi, backupApi, EXPORT_GROUPS, type ExcelSheetInfo, type ExcelSheetSelection } from '../api'
import { isUserCancellation, showDetailedError } from '../utils/errorFeedback'

const exportGroups = EXPORT_GROUPS
const exportingGroup = ref('')
const importing = ref(false)
const packageExporting = ref(false)
const packageImporting = ref(false)
const backingUp = ref(false)
const dataPath = ref('')
const backupCount = ref(10)
const backupConfig = ref<any>({})
const backups = ref<any[]>([])
const importDialogVisible = ref(false)
const importSourcePath = ref('')
const availableSheets = ref<ExcelSheetInfo[]>([])
const selectedSheets = ref<ExcelSheetSelection[]>([])

onMounted(async () => {
  try {
    const result = await settingsApi.getDataPath()
    dataPath.value = result as string
  } catch (error) {
    showDetailedError('加载数据文件路径', error)
  }
  await loadBackupConfig()
  await loadBackups()
})

async function loadBackupConfig() {
  try {
    const config = await backupApi.getConfig()
    backupConfig.value = config
    backupCount.value = config.backupCount || 10
  } catch (error) {
    showDetailedError('加载备份配置', error)
  }
}

async function loadBackups() {
  try {
    const result = await backupApi.list()
    backups.value = Array.isArray(result) ? result : []
  } catch (error) {
    showDetailedError('加载备份记录', error)
  }
}

async function handleToggleLock(index: number) {
  try {
    const result = await backupApi.toggleLock(index)
    if (result.success) {
      backups.value[index].locked = result.locked
      ElMessage.success(result.locked ? '已锁定' : '已解锁')
    }
  } catch (error) {
    showDetailedError('切换备份锁定状态', error)
  }
}

async function handleBackupNow() {
  try {
    await ElMessageBox.confirm(
      '确定要立即备份当前数据吗？',
      '确认备份',
      { type: 'info', confirmButtonText: '确定备份', cancelButtonText: '取消' }
    )
  } catch {
    return
  }
  backingUp.value = true
  try {
    const result = await backupApi.backup()
    if (result.skipped) {
      ElMessage.info(result.message || '文件内容未变化，跳过备份')
    } else {
      ElMessage.success('备份成功')
    }
    await loadBackups()
  } catch (error) {
    showDetailedError('立即备份数据', error)
  } finally {
    backingUp.value = false
  }
}

async function handleRestore(backup: any) {
  try {
    await ElMessageBox.confirm(
      `确定要恢复此备份吗？\n${backup.file_path}\n\n当前数据将自动备份。`,
      '确认恢复',
      { type: 'warning', confirmButtonText: '确定恢复', cancelButtonText: '取消' }
    )
    await backupApi.restore(backup.file_path)
    ElMessage.success('恢复成功，请重启应用')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('恢复数据备份', error)
  }
}

async function handleSelectBackupDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择备份目录'
    })
    if (!selected) return
    const previousPath = backupConfig.value.backupPath
    const previousEffectiveDir = backupConfig.value.effectiveBackupDir
    try {
      await backupApi.setConfig(backupCount.value, selected as string)
      backupConfig.value.backupPath = selected as string
      backupConfig.value.effectiveBackupDir = selected as string
      ElMessage.success('备份目录已更新')
    } catch (error) {
      backupConfig.value.backupPath = previousPath
      backupConfig.value.effectiveBackupDir = previousEffectiveDir
      throw error
    }
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('设置备份目录', error)
  }
}

async function resetBackupDir() {
  const previousPath = backupConfig.value.backupPath
  const previousEffectiveDir = backupConfig.value.effectiveBackupDir
  try {
    await backupApi.setConfig(backupCount.value, null)
    backupConfig.value.backupPath = null
    backupConfig.value.effectiveBackupDir = backupConfig.value.defaultBackupDir
    ElMessage.success('已恢复默认备份目录')
  } catch (error) {
    backupConfig.value.backupPath = previousPath
    backupConfig.value.effectiveBackupDir = previousEffectiveDir
    showDetailedError('恢复默认备份目录', error)
  }
}

async function handleExportPackage() {
  packageExporting.value = true
  try {
    const timestamp = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)
    const filePath = await save({
      defaultPath: `mold-data-package-${timestamp}.moldpkg`,
      filters: [{ name: '模具完整数据包', extensions: ['moldpkg'] }]
    })
    if (!filePath) return
    await dataApi.exportPackage(filePath)
    ElMessage.success('完整数据包导出成功')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导出完整数据包', error)
  } finally {
    packageExporting.value = false
  }
}

async function handleImportPackage() {
  packageImporting.value = true
  try {
    const filePath = await open({
      filters: [{ name: '模具完整数据包', extensions: ['moldpkg'] }],
      multiple: false,
      title: '选择完整数据包'
    })
    if (!filePath) return
    await ElMessageBox.confirm(
      '完整数据包将同时替换当前 Excel 和全部附件。当前数据集会先创建联合备份，确定继续？',
      '确认导入完整数据包',
      { type: 'warning', confirmButtonText: '确定导入', cancelButtonText: '取消' }
    )
    const result = await dataApi.importPackage(filePath as string)
    const statsText = Object.entries(result.stats)
      .map(([name, count]) => `${name}: ${count} 条`)
      .join('\n')
    await ElMessageBox.alert(
      `完整数据包导入成功！\n附件：${result.attachmentCount} 个\n\n${statsText}`,
      '导入结果',
      { type: 'success' }
    )
    window.location.reload()
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导入完整数据包', error)
  } finally {
    packageImporting.value = false
  }
}

async function handleExportGroup(groupId: string) {
  exportingGroup.value = groupId
  try {
    const timestamp = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)
    const filePath = await save({
      defaultPath: `${groupId}-${timestamp}.xlsx`,
      filters: [{ name: 'Excel 文件', extensions: ['xlsx'] }]
    })
    if (!filePath) return
    await dataApi.exportGroup(groupId, filePath)
    ElMessage.success(`「${groupId}」导出成功`)
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError(`导出「${groupId}」`, error)
  } finally {
    exportingGroup.value = ''
  }
}

async function handleImport() {
  importing.value = true
  try {
    const filePath = await open({
      filters: [{ name: 'Excel 文件', extensions: ['xlsx'] }],
      multiple: false,
      title: '选择 Excel 文件'
    })
    if (!filePath) return

    const result = await dataApi.listExcelSheets(filePath as string)
    availableSheets.value = result
    selectedSheets.value = result.filter(sheet => !sheet.systemCalculated).map(({ name, table }) => ({ name, table }))
    importSourcePath.value = filePath as string
    importDialogVisible.value = true
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('读取 Excel 工作表', error)
  } finally {
    importing.value = false
  }
}

async function confirmImport() {
  if (selectedSheets.value.length === 0) {
    ElMessage.warning('请至少勾选一个工作表')
    return
  }
  importing.value = true
  importDialogVisible.value = false
  try {
    const names = selectedSheets.value.map(selection => selection.table).join('、')
    await ElMessageBox.confirm(
      `将导入 ${selectedSheets.value.length} 个工作表：${names}\n（导入前自动创建联合备份），确定继续？`,
      '确认导入',
      { type: 'warning', confirmButtonText: '确定导入', cancelButtonText: '取消' }
    )
    const result = await dataApi.importExcelSheets(importSourcePath.value, selectedSheets.value)
    const statsText = Object.entries(result.stats)
      .map(([name, count]) => `${name}: ${count} 条`)
      .join('\n')
    await ElMessageBox.alert(`导入成功！\n\n${statsText}`, '导入结果', { type: 'success' })
    window.location.reload()
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导入 Excel 工作表', error)
  } finally {
    importing.value = false
  }
}

async function handleSelectPath() {
  try {
    const selected = await open({
      filters: [{ name: '数据库文件', extensions: ['db'] }],
      multiple: false,
      title: '选择数据文件'
    })
    if (!selected) return

    const newPath = selected as string
    await ElMessageBox.confirm(
      `确定将数据文件切换到：\n${newPath}\n\n修改后需要重启应用才能生效。`,
      '确认修改',
      { type: 'warning', confirmButtonText: '确定', cancelButtonText: '取消' }
    )

    const result = await settingsApi.setDataPath(newPath)
    if (result.success) {
      dataPath.value = result.filePath
      ElMessage.success('数据文件已更新，请重启应用')
    }
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('切换数据文件', error)
  }
}
</script>

<style scoped>
.settings-page {
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: auto;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-auto-rows: max-content;
  align-content: start;
  gap: 14px;
  padding: 1px;
  box-sizing: border-box;
  scrollbar-gutter: stable;
}

.settings-card {
  min-width: 0;
  margin: 0;
}

.settings-card :deep(.el-card__header) {
  padding: 13px 16px;
}

.settings-card :deep(.el-card__body) {
  padding: 14px 16px;
  overflow: visible;
}

.backup-records-card :deep(.el-card__body) {
  overflow: hidden;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 600;
}

.action-group {
  display: grid;
  gap: 8px;
}

.secondary-action-group {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.action-group-title {
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 600;
}

.action-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 2px;
}

.backup-action {
  margin-top: 10px;
  margin-bottom: 0;
}

.action-desc {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.65;
}

.action-desc p {
  margin: 0;
}

.path-config {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
}

.current-path {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.label {
  flex-shrink: 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.path-tag {
  min-width: 0;
  max-width: min(400px, 100%);
  overflow: hidden;
  justify-content: flex-start;
  font-family: "Cascadia Mono", "Segoe UI Mono", monospace;
  font-size: 12px;
}

.path-text {
  display: block;
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-actions {
  flex-shrink: 0;
}

.settings-card :deep(.el-form-item) {
  margin-bottom: 12px;
}

.settings-card :deep(.el-form-item:last-child) {
  margin-bottom: 0;
}

.backup-config {
  margin-bottom: 0;
}

.form-tip {
  margin-left: 10px;
  color: var(--text-muted);
  font-size: 12px;
}

.backup-path-row {
  min-width: 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.backup-path-row .path-tag {
  flex: 1 1 220px;
}

.backup-policy-tag {
  margin-left: 8px;
}

.refresh-button {
  margin-left: auto;
}

.backup-table-scroll {
  width: 100%;
  overflow: auto;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
}

.backup-table {
  width: 100%;
  min-width: 600px;
}

.backup-table-scroll :deep(.vxe-table) {
  width: 100%;
  min-width: 600px;
}

.backup-table-scroll :deep(.vxe-table--body-wrapper) {
  overflow-x: visible !important;
}

.backup-table-scroll :deep(.backup-operation-column) {
  border-left-color: var(--border);
}

.backup-table-scroll.has-horizontal-overflow :deep(.backup-operation-column) {
  border-left-color: var(--border-strong);
}

.backup-empty {
  padding: 20px;
  color: var(--text-muted);
  text-align: center;
}

.import-sheet-tip {
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 10px;
}

.import-sheet-group {
  display: grid;
  grid-template-columns: 1fr;
  gap: 4px 12px;
  max-height: 320px;
  overflow: auto;
  padding: 2px;
}

.import-sheet-item {
  height: 34px;
  margin-right: 0;
}

.import-sheet-name {
  color: var(--text-primary);
}

.import-sheet-meta {
  margin-left: 8px;
  color: var(--text-muted);
  font-size: 12px;
}

.import-sheet-tag {
  margin-left: 6px;
}

.export-group-button {
  min-width: 88px;
}

@media (min-width: 1100px) {
  .backup-records-card {
    grid-column: 1 / -1;
  }
}
</style>
