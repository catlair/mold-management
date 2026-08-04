import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import VxeUI from 'vxe-pc-ui'
import 'vxe-pc-ui/lib/style.css'
import VxeUITable from 'vxe-table'
import 'vxe-table/lib/style.css'
import './assets/print.css'
import App from './App.vue'
import router from './router'
import ConfigurableTable from './components/ConfigurableTable.vue'
import ConfigurableVxeTable from './components/ConfigurableVxeTable.vue'
import { initializeTheme } from './composables/useTheme'

// 在应用挂载前同步主题，避免启动时出现浅色闪烁
initializeTheme()

const app = createApp(App)

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(ElementPlus, { locale: zhCn })
app.use(VxeUI)
app.use(VxeUITable)
app.component('ConfigurableTable', ConfigurableTable)
app.component('ConfigurableVxeTable', ConfigurableVxeTable)
app.use(router)
app.mount('#app')
