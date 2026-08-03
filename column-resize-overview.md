# 列表列宽拖动实现概览

## 已完成
- 为全部使用 `DataTable` / `ConfigurableVxeTable` 的业务列表启用表头列宽拖动。
- 普通数据列可拖动；固定操作列保持固定宽度，避免破坏右侧固定与滚动补偿。
- 调整后的列宽自动写入现有表格偏好存储，刷新或重新进入页面后仍会恢复。
- 页面管理中的“恢复默认”会连同已保存列宽一起清除，并展示当前已保存的像素宽度。
- 增大拖动热区，增加浅色/深色主题下统一的主色边界提示。

## 关键实现
- `src/components/ConfigurableTable.vue`：读取持久化宽度，控制列是否可拖动。
- `src/components/DataTable.vue`：启用列宽调整并保存主业务表宽度。
- `src/components/ConfigurableVxeTable.vue`：启用列宽调整并保存记录表、弹窗表、库存表和备份表宽度。
- `src/composables/useTablePreferences.ts`：在列偏好中增加可选 `width`。
- `src/composables/useFixedOperationColumns.ts`：列宽变化后重新计算固定表头、固定操作列和横向溢出。
- `src/views/PageManagement.vue`：补充列宽自动保存说明与当前宽度显示。
- `src/App.vue`：增强表头拖动热区和视觉反馈。

## 验证
- Vue TypeScript 检查通过。
- Vite 生产构建通过：2185 个模块，约 1.57 秒。
- `git diff --check` 通过。

## 使用方式
将鼠标移动到任意非操作列表头的右边界，出现列宽光标后左右拖动即可。若需恢复原始宽度，进入“页面管理”，对对应表格点击“恢复默认”。
