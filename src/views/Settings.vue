<template>
  <div class="settings-page">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <el-icon><Upload /></el-icon>
          <span>数据导入导出</span>
        </div>
      </template>
      <div class="action-buttons">
        <el-button type="primary" @click="handleExport" :loading="exporting">
          <el-icon><Download /></el-icon>
          导出数据
        </el-button>
        <el-button type="warning" @click="handleImport" :loading="importing">
          <el-icon><Upload /></el-icon>
          导入数据
        </el-button>
      </div>
      <div class="action-desc">
        <p>导出：将当前所有数据导出为 Excel 文件</p>
        <p>导入：从 Excel 文件导入数据（将替换当前数据，原数据会自动备份）</p>
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
          <el-tag type="info" class="path-tag">{{ dataPath || '加载中...' }}</el-tag>
        </div>
        <div class="path-actions">
          <el-button type="primary" @click="handleSelectPath">
            <el-icon><FolderOpened /></el-icon>
            选择文件
          </el-button>
        </div>
      </div>
      <div class="action-desc">
        <p>选择一个 .xlsx 文件作为数据存储位置</p>
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
          <el-form-item label="保留份数">
            <el-input-number v-model="backupCount" :min="1" :max="100" @change="saveBackupConfig" />
            <span class="form-tip">超过此数量将自动删除最早的备份</span>
          </el-form-item>
          <el-form-item label="备份目录">
            <div class="backup-path-row">
              <el-tag type="info" class="path-tag">{{ backupConfig.effectiveBackupDir || '加载中...' }}</el-tag>
              <el-button size="small" @click="handleSelectBackupDir">更改</el-button>
              <el-button size="small" type="info" @click="resetBackupDir">恢复默认</el-button>
            </div>
          </el-form-item>
          <el-form-item label="备份策略">
            <el-tag type="success">启动时自动备份</el-tag>
            <el-tag type="success" style="margin-left: 8px">退出时自动备份</el-tag>
          </el-form-item>
        </el-form>
      </div>
      <div class="action-buttons" style="margin-top: 16px">
        <el-button type="primary" @click="handleBackupNow" :loading="backingUp">
          <el-icon><CopyDocument /></el-icon>
          立即备份
        </el-button>
      </div>
    </el-card>

    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>备份记录</span>
          <el-button size="small" style="margin-left: auto" @click="loadBackups">刷新</el-button>
        </div>
      </template>
      <el-table :data="backups" style="width: 100%" max-height="400">
        <el-table-column prop="backup_time" label="备份时间" width="180" />
        <el-table-column prop="backup_reason" label="备份原因" width="120" />
        <el-table-column prop="backup_md5" label="MD5" min-width="200" show-overflow-tooltip />
        <el-table-column label="锁定" width="80" align="center">
          <template #default="{ row, $index }">
            <el-button size="small" :type="row.locked ? 'warning' : 'info'" link @click="handleToggleLock($index)">
              <el-icon><Lock v-if="row.locked" /><Unlock v-else /></el-icon>
            </el-button>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="warning" @click="handleRestore(row)">恢复</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div v-if="backups.length === 0" style="text-align: center; color: #909399; padding: 20px">
        暂无备份记录
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { dataApi, settingsApi, backupApi } from '../api'

const exporting = ref(false)
const importing = ref(false)
const backingUp = ref(false)
const dataPath = ref('')
const backupCount = ref(10)
const backupConfig = ref<any>({})
const backups = ref<any[]>([])

onMounted(async () => {
  try {
    const result = await settingsApi.getDataPath()
    dataPath.value = result as string
  } catch (error: any) {
    console.error('获取数据路径失败:', error)
  }
  await loadBackupConfig()
  await loadBackups()
})

async function loadBackupConfig() {
  try {
    const config = await backupApi.getConfig()
    backupConfig.value = config
    backupCount.value = config.backupCount || 10
  } catch (error: any) {
    console.error('获取备份配置失败:', error)
  }
}

async function loadBackups() {
  try {
    const result = await backupApi.list()
    backups.value = Array.isArray(result) ? result : []
  } catch (error: any) {
    console.error('获取备份列表失败:', error)
  }
}

async function handleToggleLock(index: number) {
  try {
    const result = await backupApi.toggleLock(index)
    if (result.success) {
      backups.value[index].locked = result.locked
      ElMessage.success(result.locked ? '已锁定' : '已解锁')
    }
  } catch (error: any) {
    ElMessage.error('操作失败: ' + (error.message || error))
  }
}

async function saveBackupConfig() {
  try {
    await backupApi.setConfig(backupCount.value, backupConfig.value.backupPath || null)
    ElMessage.success('备份配置已保存')
  } catch (error: any) {
    ElMessage.error('保存失败: ' + (error.message || error))
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
  } catch (error: any) {
    ElMessage.error('备份失败: ' + (error.message || error))
  } finally {
    backingUp.value = false
  }
}

async function handleRestore(backup: any) {
  try {
    await ElMessageBox.confirm(
      `确定要恢复此备份吗？\n${backup.name}\n\n当前数据将自动备份。`,
      '确认恢复',
      { type: 'warning', confirmButtonText: '确定恢复', cancelButtonText: '取消' }
    )
    await backupApi.restore(backup.path)
    ElMessage.success('恢复成功，请重启应用')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('恢复失败: ' + (error.message || error))
    }
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
    backupConfig.value.backupPath = selected as string
    backupConfig.value.effectiveBackupDir = selected as string
    await backupApi.setConfig(backupCount.value, selected as string)
    ElMessage.success('备份目录已更新')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('设置失败: ' + (error.message || error))
    }
  }
}

async function resetBackupDir() {
  backupConfig.value.backupPath = null
  backupConfig.value.effectiveBackupDir = backupConfig.value.defaultBackupDir
  await backupApi.setConfig(backupCount.value, null)
  ElMessage.success('已恢复默认备份目录')
}

async function handleExport() {
  exporting.value = true
  try {
    const result = await dataApi.exportData()
    const bytes = Uint8Array.from(atob(result.data), c => c.charCodeAt(0))

    const filePath = await save({
      defaultPath: result.filename,
      filters: [{ name: 'Excel 文件', extensions: ['xlsx'] }]
    })
    if (!filePath) return

    await writeFile(filePath, bytes)
    ElMessage.success('导出成功')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('导出失败: ' + (error.message || error))
    }
  } finally {
    exporting.value = false
  }
}

async function handleImport() {
  importing.value = true
  try {
    const filePath = await open({
      filters: [{ name: 'Excel 文件', extensions: ['xlsx'] }],
      multiple: false
    })
    if (!filePath) return

    await ElMessageBox.confirm(
      '导入将替换当前所有数据（原数据会自动备份），确定继续？',
      '确认导入',
      { type: 'warning', confirmButtonText: '确定导入', cancelButtonText: '取消' }
    )

    const fileBuffer = await fetchFileAsBase64(filePath as string)
    const result = await dataApi.importData(fileBuffer)

    const statsText = Object.entries(result.stats)
      .map(([name, count]) => `${name}: ${count} 条`)
      .join('\n')

    await ElMessageBox.alert(
      `导入成功！\n\n${statsText}`,
      '导入结果',
      { type: 'success' }
    )

    window.location.reload()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('导入失败: ' + (error.message || error))
    }
  } finally {
    importing.value = false
  }
}

async function fetchFileAsBase64(filePath: string): Promise<string> {
  const bytes = await readFile(filePath)
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

async function handleSelectPath() {
  try {
    const selected = await open({
      filters: [{ name: 'Excel 文件', extensions: ['xlsx'] }],
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
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('设置失败: ' + (error.message || error))
    }
  }
}
</script>

<style scoped>
.settings-page {
  max-width: 800px;
  margin: 0 auto;
}

.settings-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.action-buttons {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.action-desc {
  color: #909399;
  font-size: 13px;
  line-height: 1.8;
}

.action-desc p {
  margin: 0;
}

.path-config {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.current-path {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label {
  color: #606266;
  font-size: 14px;
}

.path-tag {
  font-family: monospace;
  font-size: 13px;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-actions {
  flex-shrink: 0;
}

.backup-config {
  margin-bottom: 0;
}

.form-tip {
  color: #909399;
  font-size: 12px;
  margin-left: 12px;
}

.backup-path-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
