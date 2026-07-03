<template>
  <el-container class="app-container">
    <el-aside :width="isCollapse ? '64px' : '200px'" class="app-aside">
      <div class="logo" :class="{ 'logo-collapse': isCollapse }">
        <img v-if="!isCollapse" src="./assets/logo.svg" alt="" class="logo-icon" />
        <h1 v-show="!isCollapse">模具管理</h1>
        <h1 v-show="isCollapse">模</h1>
      </div>
      <el-scrollbar>
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
            <el-menu-item index="/inventory">
              <el-icon><DataAnalysis /></el-icon>
              <template #title>库存汇总</template>
            </el-menu-item>
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
.app-container {
  height: 100vh;
}

.app-aside {
  background: linear-gradient(180deg, #2b3a4e 0%, #243243 100%);
  overflow: hidden;
  transition: width 0.3s ease-in-out;
  position: relative;
  display: flex;
  flex-direction: column;
  will-change: width;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.logo-collapse {
  justify-content: center;
}

.logo-icon {
  width: 28px;
  height: 28px;
}

.logo h1 {
  color: #fff;
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  letter-spacing: 2px;
}

.el-menu-vertical {
  border-right: none;
  background: transparent;
}

.el-menu-vertical:not(.el-menu--collapse) {
  width: 200px;
}

.el-menu-vertical .el-menu-item {
  color: #bfcbd9;
  height: 48px;
  line-height: 48px;
  border-radius: 0;
  margin: 2px 8px;
  border-radius: 8px;
  transition: all 0.2s;
}

.el-menu-vertical .el-menu-item:hover {
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
}

.el-menu-vertical .el-menu-item.is-active {
  background: rgba(64, 158, 255, 0.2);
  color: #409eff;
  font-weight: 500;
}

.menu-group-title {
  font-size: 12px;
  color: #8a919e;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.collapse-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #8a919e;
  background: rgba(0, 0, 0, 0.2);
  transition: color 0.3s;
  flex-shrink: 0;
}

.collapse-btn:hover {
  color: #409eff;
  background: rgba(64, 158, 255, 0.1);
}

.app-main {
  background: #f0f2f5;
  padding: 20px;
  overflow: auto;
}

/* 页面过渡动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* 全局表格样式 */
.el-table {
  border-radius: 8px;
  overflow: hidden;
}

.el-table .el-table__cell {
  text-align: center;
}

.el-table th.el-table__cell {
  background: #f5f7fa !important;
  color: #303133 !important;
  font-weight: 600;
  font-size: 13px;
}

.el-table .el-table__body tr:nth-child(even) td.el-table__cell {
  background: #fafafa !important;
}

.el-table .el-table__body tr:hover td.el-table__cell {
  background: #ecf5ff !important;
}

/* 全局卡片样式 */
.el-card {
  border-radius: 12px;
  border: none;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.el-card__header {
  border-bottom: 1px solid #f0f0f0;
  padding: 16px 20px;
}

/* 全局按钮样式 */
.el-button {
  border-radius: 8px;
  font-weight: 500;
}

/* 全局标签样式 */
.el-tag {
  border-radius: 6px;
}

/* 对话框美化 */
.el-dialog {
  border-radius: 12px;
  overflow: hidden;
}

.el-dialog__header {
  border-bottom: 1px solid #f0f0f0;
  padding: 16px 20px;
  margin: 0;
}

.el-dialog__body {
  padding: 24px 20px;
}

/* 表单项优化 */
.el-form-item__label {
  font-weight: 500;
  color: #303133;
}

/* 页面容器通用样式 */
.page-container {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

/* 操作按钮区 */
.action-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

/* 表格上方操作栏 */
.tab-header {
  margin-bottom: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
