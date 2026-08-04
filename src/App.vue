<template>
  <el-container class="app-container">
    <el-aside :width="isCollapse ? '56px' : '180px'" class="app-aside">
      <div class="logo" :class="{ 'logo-collapse': isCollapse }">
        <img v-if="!isCollapse" src="./assets/logo.svg" alt="" class="logo-icon" />
        <span v-show="!isCollapse" class="logo-text">模具管理</span>
      </div>
      <div class="search-wrapper" :class="{ 'search-collapsed': isCollapse }">
        <GlobalSearch :collapsed="isCollapse" />
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
            <el-menu-item index="/inventory">
              <el-icon><DataAnalysis /></el-icon>
              <template #title>库存汇总</template>
            </el-menu-item>
          </el-menu-item-group>
          <el-menu-item-group>
            <template #title><span v-show="!isCollapse" class="menu-group-title">系统功能</span></template>
            <el-menu-item index="/configuration-management">
              <el-icon><Operation /></el-icon>
              <template #title>配置管理</template>
            </el-menu-item>
            <el-menu-item index="/settings">
              <el-icon><Setting /></el-icon>
              <template #title>数据与备份</template>
            </el-menu-item>
          </el-menu-item-group>
        </el-menu>
      </el-scrollbar>
      <ThemeToggle :collapsed="isCollapse" />
      <button
        class="collapse-btn"
        type="button"
        :aria-label="isCollapse ? '展开侧边栏' : '收起侧边栏'"
        :title="isCollapse ? '展开侧边栏' : '收起侧边栏'"
        @click="isCollapse = !isCollapse"
      >
        <el-icon>
          <DArrowLeft v-if="!isCollapse" />
          <DArrowRight v-else />
        </el-icon>
      </button>
    </el-aside>
    <el-main class="app-main">
      <router-view v-slot="{ Component }">
        <transition name="fade-slide" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </el-main>

    <!-- 全局全屏退出按钮（所有页面共用） -->
    <el-button v-if="isFullscreen" class="fullscreen-exit-btn" type="danger" circle @click="toggleFullscreen">
      <el-icon :size="20"><Close /></el-icon>
    </el-button>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, provide } from 'vue'
import { useRoute } from 'vue-router'
import GlobalSearch from './components/GlobalSearch.vue'
import ThemeToggle from './components/ThemeToggle.vue'
import { useFullscreen } from './composables/useFullscreen'
import { useFixedOperationColumns } from './composables/useFixedOperationColumns'

// 全屏能力全局注入：页面通过 inject 获取，无需各自实现
const { isFullscreen, toggleFullscreen } = useFullscreen()
useFixedOperationColumns()
provide('fullscreen', { isFullscreen, toggleFullscreen })

const route = useRoute()
const activeMenu = computed(() => route.path)
const isCollapse = ref(false)
// 侧栏宽度提供给应用内覆盖层（如附件预览），使其精确铺满主内容区域
provide('appSidebarWidth', computed(() => isCollapse.value ? 56 : 180))

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
  color-scheme: light;
  --primary: #315ea8;
  --primary-light: #4774bc;
  --sidebar-bg: #eef1f5;
  --sidebar-hover: #e1e6ed;
  --sidebar-active: rgba(49, 94, 168, 0.12);
  --sidebar-text: #485365;
  --sidebar-text-active: var(--primary);
  --bg: #f4f6f8;
  --card-bg: #ffffff;
  --surface-muted: #f8fafc;
  --surface-hover: #eef3f8;
  --table-stripe-bg: #fbfcfd;
  --switch-off-bg: #aab3c0;
  --switch-disabled-bg: #d8dde4;
  --text-primary: #1d2735;
  --text-secondary: #667085;
  --text-muted: #8a94a4;
  --border: #dfe4ea;
  --border-strong: #cdd4dd;
  --focus-ring: #315ea8;
  --scrollbar-track: #edf0f4;
  --scrollbar-thumb: #aab3c0;
  --scrollbar-thumb-hover: #7f8a99;
  --shadow-card: 0 1px 3px rgba(23, 34, 51, 0.06), 0 10px 28px rgba(23, 34, 51, 0.04);
  --el-color-primary: #315ea8;
  --el-bg-color: var(--card-bg);
  --el-bg-color-overlay: var(--card-bg);
  --el-fill-color-blank: var(--card-bg);
  --el-text-color-primary: var(--text-primary);
  --el-text-color-regular: var(--text-secondary);
  --el-border-color: var(--border);
  --el-border-color-light: var(--border);
  --el-fill-color-light: var(--surface-muted);
  --el-mask-color: rgba(15, 23, 42, 0.52);
  --vxe-ui-font-color: var(--text-primary);
  --vxe-ui-font-primary-color: var(--primary);
  --vxe-ui-font-secondary-color: var(--text-secondary);
  --vxe-ui-layout-background-color: var(--card-bg);
  --vxe-ui-table-header-font-color: var(--text-primary);
  --vxe-ui-table-header-background-color: var(--surface-muted);
  --vxe-ui-table-border-color: var(--border);
  --vxe-ui-table-row-hover-background-color: var(--surface-hover);
  --vxe-ui-table-row-striped-background-color: #fbfcfd;
  --vxe-ui-table-row-hover-striped-background-color: var(--surface-hover);
  --vxe-ui-loading-background-color: rgba(255, 255, 255, 0.76);
}

:root.dark {
  color-scheme: dark;
  --primary: #7ca1dc;
  --primary-light: #99b8e6;
  --sidebar-bg: #171c24;
  --sidebar-hover: #222a35;
  --sidebar-active: rgba(124, 161, 220, 0.16);
  --sidebar-text: #b8c1cf;
  --sidebar-text-active: var(--primary);
  --bg: #10141a;
  --card-bg: #1a2029;
  --surface-muted: #202731;
  --surface-hover: #27313d;
  --table-stripe-bg: #1d242d;
  --switch-off-bg: #4e5a69;
  --switch-disabled-bg: #343e4b;
  --text-primary: #edf1f7;
  --text-secondary: #b7c0ce;
  --text-muted: #8995a6;
  --border: #303946;
  --border-strong: #3d4857;
  --focus-ring: #90afe0;
  --scrollbar-track: #171d25;
  --scrollbar-thumb: #4e5a69;
  --scrollbar-thumb-hover: #6d7a8b;
  --shadow-card: 0 1px 3px rgba(0, 0, 0, 0.28), 0 14px 34px rgba(0, 0, 0, 0.18);
  --el-color-primary: #7ca1dc;
  --el-bg-color: var(--card-bg);
  --el-bg-color-overlay: #202731;
  --el-fill-color-blank: var(--card-bg);
  --el-fill-color: #242c37;
  --el-fill-color-light: var(--surface-muted);
  --el-fill-color-lighter: #232b35;
  --el-fill-color-extra-light: #202731;
  --el-text-color-primary: var(--text-primary);
  --el-text-color-regular: var(--text-secondary);
  --el-text-color-secondary: var(--text-muted);
  --el-border-color: var(--border);
  --el-border-color-light: var(--border);
  --el-border-color-lighter: #2a333f;
  --el-border-color-extra-light: #252d37;
  --el-mask-color: rgba(0, 0, 0, 0.68);
  --vxe-ui-font-color: var(--text-primary);
  --vxe-ui-font-primary-color: var(--primary);
  --vxe-ui-font-secondary-color: var(--text-secondary);
  --vxe-ui-layout-background-color: var(--card-bg);
  --vxe-ui-table-header-font-color: var(--text-primary);
  --vxe-ui-table-header-background-color: var(--surface-muted);
  --vxe-ui-table-footer-background-color: var(--surface-muted);
  --vxe-ui-table-border-color: var(--border);
  --vxe-ui-table-column-hover-background-color: var(--surface-hover);
  --vxe-ui-table-row-hover-background-color: var(--surface-hover);
  --vxe-ui-table-row-striped-background-color: #1d242d;
  --vxe-ui-table-row-hover-striped-background-color: var(--surface-hover);
  --vxe-ui-loading-background-color: rgba(26, 32, 41, 0.78);
  --vxe-ui-base-popup-border-color: var(--border);
  --vxe-ui-input-border-color: var(--border-strong);
  --vxe-ui-input-disabled-background-color: #202731;
}

html,
body,
#app {
  margin: 0;
  min-width: 900px;
  min-height: 100%;
  background: var(--bg);
  color: var(--text-primary);
}

html {
  transition: background-color 0.2s ease, color 0.2s ease;
}

body {
  font-family: "Segoe UI", "Microsoft YaHei UI", "Microsoft YaHei", sans-serif;
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}

.app-container {
  height: 100vh;
}

.app-aside {
  background: var(--sidebar-bg);
  overflow: hidden;
  transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.2s ease, border-color 0.2s ease;
  position: relative;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
}

.logo {
  height: 56px;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 0 16px;
  gap: 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.logo-collapse {
  padding: 0;
  justify-content: center;
}

.logo-icon {
  width: 32px !important;
  height: 32px !important;
  flex-shrink: 0;
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

.search-wrapper.search-collapsed {
  padding: 0 8px 8px;
}

.el-menu-vertical {
  border-right: none;
  background: transparent;
  padding: 8px 0;
}

.el-menu-vertical:not(.el-menu--collapse) {
  width: 180px;
}

.el-menu-vertical.el-menu--collapse {
  width: 56px;
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

/* 折叠态图标居中：Element Plus 通过 .el-menu-tooltip__trigger 渲染折叠菜单项，
   其默认 padding:0 20px + 左对齐会与自定义 56px 侧栏宽度错位，导致图标偏右。
   折叠时取消 padding 并让内容居中；展开态不受影响。 */
.el-menu-vertical.el-menu--collapse .el-menu-item .el-menu-tooltip__trigger {
  padding: 0;
  justify-content: center;
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
  color: var(--text-muted);
  letter-spacing: 1.5px;
  font-weight: 600;
}

/* 折叠态隐藏分组标题占位（标题文字已隐藏，但容器默认 padding 会留出空白） */
.el-menu-vertical.el-menu--collapse .el-menu-item-group__title {
  height: 0;
  padding: 0;
  font-size: 0;
  line-height: 0;
  overflow: hidden;
}

.collapse-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--sidebar-text);
  background: transparent;
  border: 0;
  border-top: 1px solid var(--border);
  transition: color 0.2s ease, background-color 0.2s ease, transform 0.2s ease;
  flex-shrink: 0;
}

.collapse-btn:hover {
  color: var(--primary-light);
  background: var(--sidebar-hover);
}

.collapse-btn:active {
  transform: scale(0.98);
}

.collapse-btn:focus-visible,
.el-button:focus-visible,
.el-link:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.app-main {
  background: var(--bg);
  padding: 24px;
  overflow: hidden;
}
.app-main.el-main {
  height: 100vh;
  overflow: hidden;
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
  --el-table-border-color: var(--border);
  --el-table-border: 1px solid var(--border);
  --el-table-bg-color: var(--card-bg);
  --el-table-tr-bg-color: var(--card-bg);
  --el-table-expanded-cell-bg-color: var(--card-bg);
  --el-table-header-bg-color: var(--surface-muted);
  --el-table-fixed-box-shadow: var(--shadow-card);
  --el-table-row-hover-bg-color: var(--surface-hover);
  --el-table-current-row-bg-color: var(--surface-hover);
  --el-table-header-text-color: var(--text-primary);
  --el-table-text-color: var(--text-secondary);
  border-radius: 10px;
  background: var(--card-bg);
}

/* DataTable 内部表格无滚动条（滚动由 dt-wrap 承担），这里仅兜底隐藏悬浮条 */
.el-table .el-scrollbar__bar.is-horizontal { display: none !important; }

/* 表头固定在滚动容器顶部：滚动同步器写入纵向偏移，兼容 vxe 包装层和 WebView2 */
:is(.dt-wrap, .record-table-scroll, .backup-table-scroll, .stock-table-shell) .vxe-table--header-wrapper {
  position: relative !important;
  z-index: 10;
  transform: translateY(var(--table-header-offset, 0));
  background: var(--surface-muted);
  box-shadow: 0 1px 0 var(--border-strong), 0 8px 14px -14px rgba(23, 34, 51, 0.5);
  will-change: transform;
}

:is(.dt-wrap, .record-table-scroll, .backup-table-scroll, .stock-table-shell) .vxe-header--column {
  background: var(--surface-muted) !important;
}

/* 列宽拖动：放大表头边界热区，并用主题主色提供明确反馈。
   z-index 需高于操作列（表头 z-index: 12），否则横向溢出固定操作列时会盖住
   倒数第二列右边缘伸出的拖拽热区，导致该处无法拖动列宽。 */
.vxe-table .vxe-header--column > .vxe-cell--col-resizable {
  right: -5px;
  width: 10px;
  z-index: 20;
}

.vxe-table .vxe-header--column > .vxe-cell--col-resizable::before {
  opacity: 0;
}

.vxe-table .vxe-header--column > .vxe-cell--col-resizable::after {
  position: absolute;
  top: 22%;
  right: 4px;
  bottom: 22%;
  width: 2px;
  height: auto;
  background: var(--el-color-primary);
  border-radius: 2px;
  opacity: 0;
  transition: opacity 140ms ease;
}

.vxe-table .vxe-header--column > .vxe-cell--col-resizable:hover::after {
  opacity: 0.72;
}

.vxe-table .vxe-table--resizable-col-bar::before {
  background-color: var(--el-color-primary);
}

/* 操作列固定在右侧：滚动同步器写入偏移，兼容 DataTable 外层滚动和记录表滚动容器 */
.vxe-table .vxe-header--column.operation-column,
.vxe-table .vxe-body--column.operation-column {
  position: relative;
  z-index: 4;
  transform: translateX(var(--operation-column-offset, 0));
  border-left: 1px solid var(--border-strong);
  background: var(--card-bg) !important;
  will-change: transform;
}

.has-horizontal-overflow .vxe-table .vxe-header--column.operation-column,
.has-horizontal-overflow .vxe-table .vxe-body--column.operation-column {
  box-shadow: -8px 0 14px -12px rgba(23, 34, 51, 0.42);
}

.vxe-table .vxe-header--column.operation-column {
  z-index: 12;
  background: var(--surface-muted) !important;
}

.vxe-table .vxe-body--row.row--stripe .vxe-body--column.operation-column {
  background: var(--vxe-ui-table-row-striped-background-color, var(--surface-muted)) !important;
}

/* vxe-table 以 row--hover 表示当前悬停行；同时保留 :hover 兜底，兼容 WebView2 */
.vxe-table .vxe-body--row.row--hover > .vxe-body--column,
.vxe-table .vxe-body--row:hover > .vxe-body--column {
  background: var(--surface-hover) !important;
}

.vxe-table .vxe-body--row.row--hover > .vxe-body--column.operation-column,
.vxe-table .vxe-body--row:hover > .vxe-body--column.operation-column {
  background: var(--surface-hover) !important;
}

/* 全屏模式 */
.page-container.is-fullscreen { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2000; background: var(--bg); padding: 0; overflow: auto; }
.page-container.is-fullscreen > .el-card { height: 100%; display: flex; flex-direction: column; margin: 0; border: none; border-radius: 0; box-shadow: none; }
.page-container.is-fullscreen > .el-card > .el-card__header { display: none; }
.page-container.is-fullscreen > .el-card > .el-card__body { flex: 1; overflow: hidden; padding: 12px; }

/* 非全屏模式：祖先链 flex 撑开高度，避免 DataTable 撑爆整页被 overflow:hidden 裁掉 */
.app-main { display: flex; flex-direction: column; }
.page-container { display: flex; flex-direction: column; min-height: 0; flex: 1; }
.page-container > .el-card { display: flex; flex-direction: column; min-height: 0; flex: 1; }
.page-container > .el-card > .el-card__body { display: flex; flex-direction: column; min-height: 0; flex: 1; padding: 12px; overflow: hidden; }
/* tabs 页：el-tabs 内容区也要撑满 */
.page-container > .el-card > .el-card__body > .el-tabs { display: flex; flex-direction: column; min-height: 0; flex: 1; }
.page-container > .el-card > .el-card__body > .el-tabs > .el-tabs__content { display: flex; flex-direction: column; min-height: 0; flex: 1; }
.page-container > .el-card > .el-card__body > .el-tabs > .el-tabs__content > .el-tab-pane { display: flex; flex-direction: column; min-height: 0; flex: 1; }

/* 全屏退出按钮（所有页面共用） */
.fullscreen-exit-btn {
  position: fixed;
  top: 12px;
  right: 12px;
  z-index: 2001;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
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
  background: var(--surface-muted) !important;
  color: var(--text-primary) !important;
  font-weight: 600;
  font-size: 13px;
  border-bottom: 2px solid var(--border) !important;
}

.el-table .el-table__body tr:nth-child(even) td.el-table__cell {
  background: var(--table-stripe-bg) !important;
}

.el-table .el-table__body tr:hover td.el-table__cell,
.el-table .el-table__body tr.hover-row td.el-table__cell {
  background: var(--surface-hover) !important;
}

/* 详细错误通知：保留换行并显示后端原因与处理建议 */
.detailed-error-notification {
  width: min(480px, calc(100vw - 32px));
}

.detailed-error-notification .el-notification__content {
  color: var(--text-secondary);
  white-space: pre-line;
  overflow-wrap: anywhere;
  line-height: 1.6;
  text-align: left;
}

/* 卡片样式 */
.el-card {
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--card-bg);
  box-shadow: var(--shadow-card);
  transition: background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.el-card__header {
  border-bottom: 1px solid var(--border);
  padding: 18px 24px;
}

.el-card__body {
  overflow: hidden;
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
  color: var(--text-primary);
}

/* 页面容器 */
.page-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.page-container > .el-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.page-container > .el-card > .el-card__body {
  flex: 1;
  overflow: hidden;
  padding: 12px 24px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* tabs 页面：让 tabs 撑满内容区，表格随容器自适应高度 */
.page-container > .el-card > .el-card__body > .el-tabs {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.page-container > .el-card > .el-card__body > .el-tabs > .el-tabs__content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.page-container > .el-card > .el-card__body > .el-tabs > .el-tabs__content > .el-tab-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
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
  flex-shrink: 0;
}

.record-pane > .tab-header,
.association-pane > .tab-header {
  min-height: 58px;
  margin-bottom: 14px;
  padding: 10px 12px 10px 16px;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  box-sizing: border-box;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface-muted);
  box-shadow: inset 0 1px 0 color-mix(in srgb, var(--card-bg) 85%, transparent);
}

.record-heading {
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.record-heading__title {
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 650;
  line-height: 1.35;
  letter-spacing: 0.01em;
}

.record-heading__count {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
  white-space: nowrap;
}

.tab-header > .el-button {
  flex-shrink: 0;
}

/* 记录/关联 tabs：仅中间表格区滚动，工具栏与分页始终留在可视区 */
.record-pane,
.association-pane {
  overflow: hidden;
}

.record-table-scroll {
  flex: 1;
  min-height: 0;
  width: 100%;
  padding: 8px;
  overflow: auto;
  box-sizing: border-box;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  border: 1px solid var(--border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--surface-muted) 70%, var(--card-bg));
  box-shadow: inset 0 1px 2px rgba(23, 34, 51, 0.04);
}

.record-table-scroll > .record-table {
  min-width: 100%;
  background: var(--card-bg);
}

.record-table .vxe-table--body-wrapper .vxe-body--column .vxe-cell,
.record-table .vxe-table--body-wrapper .vxe-body--column .vxe-cell--wrapper {
  height: auto;
  overflow: visible;
  text-overflow: clip;
  white-space: normal;
  overflow-wrap: anywhere;
  word-break: break-word;
  line-height: 1.55;
}

.record-table .vxe-table--body-wrapper .vxe-body--column .vxe-cell {
  padding-top: 11px;
  padding-bottom: 11px;
}

.record-table .vxe-table--header-wrapper .vxe-header--column {
  font-weight: 600;
}

.record-pagination-bar {
  min-height: 48px;
  margin-top: 12px;
  padding: 12px 4px 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
  border-top: 1px solid var(--border);
}

.record-pagination {
  flex-shrink: 0;
  justify-content: flex-end;
}

.record-pagination :is(button, .el-select__wrapper):focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.record-table-scroll::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

.record-table-scroll::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
  border-radius: 5px;
}

.record-table-scroll::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border: 2px solid var(--scrollbar-track);
  border-radius: 5px;
}

.record-table-scroll::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

/* 搜索跳转高亮动画 */
.highlight-flash {
  animation: flash-highlight 2s ease;
}
@keyframes flash-highlight {
  0%, 100% { background: inherit; }
  10%, 30%, 50% { background: var(--sidebar-active) !important; }
  20%, 40% { background: color-mix(in srgb, var(--primary) 22%, var(--card-bg)) !important; }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    scroll-behavior: auto !important;
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
