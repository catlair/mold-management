# 表格整行 Hover 修复概览

## 问题根因

- 固定操作列有单独的原生 `:hover` 强制背景规则，因此鼠标经过时会变色。
- 普通列依赖 vxe-table 的 `.row--hover` 状态类，但现有表格没有统一开启 `row-config.isHover`，所以普通列没有进入行悬停状态。

## 修复内容

- `src/components/DataTable.vue`
  - 主业务表格的 `row-config` 增加 `isHover: true`。
- `src/components/ConfigurableVxeTable.vue`
  - 所有记录表、关联表、库存表和备份表统一合并调用方 `rowConfig`，并强制启用 `isHover: true`。
- `src/App.vue`
  - 全局 Hover 规则从只覆盖操作列，扩展为覆盖悬停行的所有 `.vxe-body--column`。
  - 同时支持 vxe-table 的 `.row--hover` 和原生 `:hover`，兼容 WebView2。
  - 使用 `!important` 保证 Hover 优先于普通行、斑马行和固定操作列背景。

## 验证结果

- 整行 Hover 规则静态核验通过。
- Vue TypeScript 检查通过。
- Vite 生产构建通过：2185 个模块，耗时 1.53 秒。
- `git diff --check` 通过；仅有既有的 LF/CRLF 行尾提示。

## 影响范围

- 主业务表格。
- 入库、领用和使用记录表。
- 关联记录与关联详情表。
- 库存汇总表。
- 备份记录表。
- 浅色和深色主题。
