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
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <template #title>配置</template>
        </el-menu-item>
      </el-menu>
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
  overflow: hidden;
  transition: width 0.3s ease-in-out;
  position: relative;
  display: flex;
  flex-direction: column;
  will-change: width;
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
  will-change: transform;
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

/* 空状态 */
.empty-state {
  padding: 40px 0;
  text-align: center;
}

/* 页面加载动画 */
.page-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
  color: #909399;
  font-size: 14px;
}

/* 卡片样式优化 */
.el-card {
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.06);
}

/* 按钮圆角 */
.el-button {
  border-radius: 6px;
}

/* 侧边栏图标对齐 */
.el-menu-item .el-icon {
  margin-right: 8px;
}
</style>
