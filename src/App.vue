<template>
  <el-container class="app-container">
    <el-aside :width="isCollapse ? '56px' : '180px'" class="app-aside">
      <div class="logo" :class="{ 'logo-collapse': isCollapse }">
        <img v-if="!isCollapse" src="./assets/logo.svg" alt="" class="logo-icon" />
        <span v-show="!isCollapse" class="logo-text">模具管理</span>
      </div>
      <div v-show="!isCollapse" class="search-wrapper">
        <GlobalSearch />
      </div>
      <el-scrollbar class="menu-scrollbar">
        <el-menu
          :default-active="activeMenu"
          :collapse="isCollapse"
          class="el-menu-vertical"
          router
        >
          <el-menu-item-group>
            <template #title><span v-show="!isCollapse" class="menu-group-title">数据管理</span></template>
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
          </el-menu-item-group>
          <el-menu-item-group>
            <template #title><span v-show="!isCollapse" class="menu-group-title">系统功能</span></template>
            <el-menu-item index="/settings">
              <el-icon><Setting /></el-icon>
              <template #title>配置</template>
            </el-menu-item>
          </el-menu-item-group>
        </el-menu>
      </el-scrollbar>
      <div class="collapse-btn" @click="isCollapse = !isCollapse">
        <el-icon>
          <DArrowLeft v-if="!isCollapse" />
          <DArrowRight v-else />
        </el-icon>
      </div>
    </el-aside>
    <el-main class="app-main">
      <router-view v-slot="{ Component }">
        <transition name="fade-slide" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import GlobalSearch from './components/GlobalSearch.vue'

const route = useRoute()
const activeMenu = computed(() => route.path)
const isCollapse = ref(false)

onMounted(() => {
  const loading = document.getElementById('app-loading')
  if (loading) {
    loading.style.transition = 'opacity 0.3s ease'
    loading.style.opacity = '0'
    setTimeout(() => loading.remove(), 300)
  }
})
</script>

<style>
:root {
  --primary: #4f6ef7;
  --primary-light: #6b8aff;
  --sidebar-bg: #f0f2f8;
  --sidebar-hover: #e0e4f0;
  --sidebar-active: rgba(79, 110, 247, 0.12);
  --sidebar-text: #4a5068;
  --sidebar-text-active: var(--primary);
  --bg: #f5f6fa;
  --card-bg: #fff;
  --border: #ebeef5;
}

.app-container {
  height: 100vh;
}

.app-aside {
  background: var(--sidebar-bg);
  overflow: hidden;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
}

.logo {
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.logo-collapse {
  justify-content: center;
}

.logo-icon {
  width: 32px;
  height: 32px;
}

.logo-text {
  color: var(--primary);
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 1px;
}

.menu-scrollbar {
  flex: 1;
}

.search-wrapper {
  padding: 0 10px 8px;
  flex-shrink: 0;
}

.el-menu-vertical {
  border-right: none;
  background: transparent;
  padding: 8px 0;
}

.el-menu-vertical:not(.el-menu--collapse) {
  width: 180px;
}

.el-menu-vertical .el-menu-item {
  color: var(--sidebar-text);
  height: 40px;
  line-height: 40px;
  margin: 2px 8px;
  border-radius: 6px;
  transition: all 0.2s ease;
  padding: 0 16px !important;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.el-menu-vertical .el-menu-item:hover {
  background: var(--sidebar-hover);
  color: var(--sidebar-text-active);
}

.el-menu-vertical .el-menu-item.is-active {
  background: var(--sidebar-active);
  color: var(--primary);
  font-weight: 600;
}

.el-menu-vertical .el-menu-item .el-icon {
  font-size: 18px;
}

.menu-group-title {
  font-size: 11px;
  color: #8a8fa8;
  letter-spacing: 1.5px;
  font-weight: 600;
}

.collapse-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--sidebar-text);
  border-top: 1px solid var(--border);
  transition: all 0.2s;
  flex-shrink: 0;
}

.collapse-btn:hover {
  color: var(--primary-light);
  background: var(--sidebar-hover);
}

.app-main {
  background: var(--bg);
  padding: 24px;
  overflow: auto;
}

/* 过渡动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* 表格样式 */
.el-table {
  border-radius: 10px;
}

/* 全屏模式表格高度更大 */
.page-container.is-fullscreen .el-table {
  max-height: 100% !important;
}

/* 表格水平滚动条始终可见 - 强制覆盖 JS 行为 */
.el-table .el-scrollbar__bar.is-horizontal {
  opacity: 1 !important;
  transition: none !important;
  transform: none !important;
}

.el-table .el-scrollbar__bar.is-horizontal .el-scrollbar__thumb {
  opacity: 1 !important;
  background: #b0b4bc !important;
  transition: background 0.2s !important;
}

.el-table .el-scrollbar__bar.is-horizontal .el-scrollbar__thumb:hover {
  background: #909399 !important;
}

/* 菜单隐藏滚动条 */
.menu-scrollbar .el-scrollbar__bar {
  display: none !important;
}

.el-table .el-table__cell {
  text-align: center;
  padding: 10px 0;
}

.el-table th.el-table__cell {
  background: #fafbfc !important;
  color: #303133 !important;
  font-weight: 600;
  font-size: 13px;
  border-bottom: 2px solid var(--border) !important;
}

.el-table .el-table__body tr:nth-child(even) td.el-table__cell {
  background: #fafbfc !important;
}

.el-table .el-table__body tr:hover td.el-table__cell {
  background: #f0f5ff !important;
}

/* 卡片样式 */
.el-card {
  border-radius: 12px;
  border: 1px solid var(--border);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.el-card__header {
  border-bottom: 1px solid var(--border);
  padding: 18px 24px;
}

.el-card__body {
  overflow: visible;
  padding: 20px 24px;
}

/* 按钮样式 */
.el-button {
  border-radius: 8px;
  font-weight: 500;
}

.el-button--primary {
  box-shadow: 0 2px 6px rgba(79, 110, 247, 0.25);
}

.el-button--primary:hover {
  box-shadow: 0 4px 12px rgba(79, 110, 247, 0.35);
}

/* 标签样式 */
.el-tag {
  border-radius: 6px;
  font-weight: 500;
}

/* 对话框 */
.el-dialog {
  border-radius: 12px;
  overflow: hidden;
}

.el-dialog__header {
  border-bottom: 1px solid var(--border);
  padding: 18px 24px;
  margin: 0;
}

.el-dialog__body {
  padding: 24px;
}

/* 表单项 */
.el-form-item__label {
  font-weight: 500;
  color: #303133;
}

/* 页面容器 */
.page-container {
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 17px;
  font-weight: 600;
  color: #1a1f36;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.tab-header {
  margin-bottom: 12px;
  display: flex;
  justify-content: flex-end;
}

/* el-table 水平滚动条始终可见 */
.el-table .el-scrollbar__bar.is-horizontal {
  display: block !important;
  height: 10px !important;
}
.el-table .el-scrollbar__bar.is-horizontal .el-scrollbar__thumb {
  background-color: #b0b4cc !important;
  border-radius: 5px;
  height: 8px !important;
  cursor: grab;
}
.el-table .el-scrollbar__bar.is-horizontal .el-scrollbar__thumb:hover {
  background-color: #909399 !important;
}
.el-table .el-scrollbar__wrap {
  overflow-x: auto !important;
}

/* 搜索跳转高亮动画 */
.highlight-flash {
  animation: flash-highlight 2s ease;
}
@keyframes flash-highlight {
  0%, 100% { background: inherit; }
  10%, 30%, 50% { background: #ecf5ff !important; }
  20%, 40% { background: #d9ecff !important; }
}
</style>
