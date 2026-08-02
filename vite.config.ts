import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    // 本机 safe-delete 钩子会拦截 dist 清空操作，禁用自动清空，避免构建失败
    emptyOutDir: false,
    // 桌面本地加载仍保持依赖分包，避免业务主包随表格组件持续膨胀
    chunkSizeWarningLimit: 1200,
    rolldownOptions: {
      output: {
        // 代码分割：将重量级依赖拆为独立 chunk，降低主包体积与首屏加载
        manualChunks(id: string) {
          if (id.includes('node_modules/@element-plus/icons-vue')) {
            return 'element-icons'
          }
          if (id.includes('node_modules/element-plus')) {
            return 'element-plus'
          }
          if (
            id.includes('node_modules/vxe-table') ||
            id.includes('node_modules/vxe-pc-ui') ||
            id.includes('node_modules/@vxe-ui') ||
            id.includes('node_modules/xe-utils') ||
            id.includes('node_modules/dom-zindex')
          ) {
            return 'vxe-table'
          }
          if (id.includes('node_modules/vue') || id.includes('node_modules/vue-router')) {
            return 'vue-vendor'
          }
        },
      },
    },
  },
})
