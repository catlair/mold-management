import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    // 本机 safe-delete 钩子会拦截 dist 清空操作，禁用自动清空，避免构建失败
    emptyOutDir: false,
    // element-plus 独立 chunk 约 883KB（gzip 283KB），桌面本地加载可接受，提高阈值消除警告噪音
    chunkSizeWarningLimit: 1000,
    rolldownOptions: {
      output: {
        // 代码分割：将重量级依赖拆为独立 chunk，降低主包体积与首屏加载
        manualChunks(id: string) {
          if (id.includes('node_modules/vue') || id.includes('node_modules/vue-router')) {
            return 'vue-vendor'
          }
          if (id.includes('node_modules/element-plus')) {
            return 'element-plus'
          }
          if (id.includes('node_modules/@element-plus/icons-vue')) {
            return 'element-icons'
          }
        },
      },
    },
  },
})
