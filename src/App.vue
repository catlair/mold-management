<template>
  <el-container class="app-container">
    <el-aside :width="isCollapse ? '64px' : '200px'" class="app-aside">
      <div class="logo" :class="{ 'logo-collapse': isCollapse }">
        <h1 v-show="!isCollapse">模具管理</h1>
        <h1 v-show="isCollapse">模</h1>
      </div>
      <el-menu
        :default-active="activeMenu"
        :collapse="isCollapse"
        class="el-menu-vertical"
        router
      >
        <el-menu-item index="/screw-spec">
          <el-icon><Document /></el-icon>
          <template #title>螺丝规格</template>
        </el-menu-item>
        <el-menu-item index="/punch">
          <el-icon><SetUp /></el-icon>
          <template #title>冲头管理</template>
        </el-menu-item>
        <el-menu-item index="/die">
          <el-icon><Grid /></el-icon>
          <template #title>牙板管理</template>
        </el-menu-item>
        <el-menu-item index="/belt">
          <el-icon><Connection /></el-icon>
          <template #title>皮带管理</template>
        </el-menu-item>
        <el-menu-item index="/main-mold">
          <el-icon><Box /></el-icon>
          <template #title>主模具管理</template>
        </el-menu-item>
        <el-menu-item index="/scissor">
          <el-icon><Scissor /></el-icon>
          <template #title>剪刀管理</template>
        </el-menu-item>
        <el-menu-item index="/upper-punch">
          <el-icon><Top /></el-icon>
          <template #title>上冲管理</template>
        </el-menu-item>
        <el-menu-item index="/inventory">
          <el-icon><DataAnalysis /></el-icon>
          <template #title>库存汇总</template>
        </el-menu-item>
      </el-menu>
      <div class="sidebar-actions" :class="{ collapsed: isCollapse }">
        <div class="action-btn" @click="handleExport" :title="isCollapse ? '导出数据' : ''">
          <el-icon><Download /></el-icon>
          <span v-show="!isCollapse">导出数据</span>
        </div>
        <div class="action-btn" @click="handleImport" :title="isCollapse ? '导入数据' : ''">
          <el-icon><Upload /></el-icon>
          <span v-show="!isCollapse">导入数据</span>
        </div>
      </div>
      <div class="collapse-btn" @click="isCollapse = !isCollapse">
        <el-icon>
          <DArrowLeft v-if="!isCollapse" />
          <DArrowRight v-else />
        </el-icon>
      </div>
    </el-aside>
    <el-main class="app-main">
      <router-view />
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { dataApi } from './api'

const route = useRoute()
const activeMenu = computed(() => route.path)
const isCollapse = ref(false)

async function handleExport() {
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
  }
}

async function handleImport() {
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

    // 刷新页面以加载新数据
    window.location.reload()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('导入失败: ' + (error.message || error))
    }
  }
}

// 读取本地文件为 base64（通过 Tauri fs 读取）
async function fetchFileAsBase64(filePath: string): Promise<string> {
  const bytes = await readFile(filePath)
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
}

#app {
  height: 100%;
}

.app-container {
  height: 100vh;
}

.app-aside {
  background: #304156;
  overflow: visible;
  transition: width 0.3s;
  position: relative;
  display: flex;
  flex-direction: column;
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #2b2f3a;
}

.logo h1 {
  color: #fff;
  font-size: 16px;
  white-space: nowrap;
}

.el-menu-vertical {
  border-right: none;
  background: #304156;
  flex: 1;
  overflow-y: auto;
}

.el-menu-vertical:not(.el-menu--collapse) {
  width: 200px;
}

.el-menu-vertical .el-menu-item {
  color: #bfcbd9;
}

.el-menu-vertical .el-menu-item:hover,
.el-menu-vertical .el-menu-item.is-active {
  background: #263445;
  color: #409eff;
}

.collapse-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #bfcbd9;
  background: #2b2f3a;
  transition: color 0.3s;
  flex-shrink: 0;
}

.collapse-btn:hover {
  color: #409eff;
}

.sidebar-actions {
  border-top: 1px solid #3a4a5c;
  padding: 8px 0;
}

.sidebar-actions.collapsed {
  border-top: none;
}

.sidebar-actions .action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 40px;
  padding: 0 20px;
  cursor: pointer;
  color: #bfcbd9;
  transition: all 0.3s;
  font-size: 14px;
}

.sidebar-actions .action-btn:hover {
  color: #409eff;
  background: #263445;
}

.sidebar-actions.collapsed .action-btn {
  padding: 0;
}

.app-main {
  background: #f0f2f5;
  padding: 20px;
}

/* 表格内容居中 */
.el-table .el-table__cell {
  text-align: center;
}

/* 表头背景 */
.el-table th.el-table__cell {
  background: #409eff !important;
  color: #fff !important;
  font-weight: 600;
}

/* 表格行交替颜色 */
.el-table .el-table__body tr:nth-child(even) td.el-table__cell {
  background: #f5f7fa !important;
}

/* 表格行悬停效果 */
.el-table .el-table__body tr:hover td.el-table__cell {
  background: #ecf5ff !important;
}
</style>
