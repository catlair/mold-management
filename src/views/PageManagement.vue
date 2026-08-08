<template>
  <div class="configuration-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <div class="title-block">
            <el-icon><Operation /></el-icon>
            <div>
              <strong>配置管理</strong>
              <span>统一管理业务选项与页面表格；导入导出会包含全部已注册配置。</span>
            </div>
          </div>
          <div class="header-actions">
            <el-button :loading="importing" :disabled="exporting" @click="handleImport">
              <el-icon><Upload /></el-icon>
              导入全部配置
            </el-button>
            <el-button type="primary" :loading="exporting" :disabled="importing" @click="handleExport">
              <el-icon><Download /></el-icon>
              导出全部配置
            </el-button>
          </div>
          <input
            ref="browserFileInput"
            class="browser-file-input"
            type="file"
            accept="application/json,.json"
            @change="handleBrowserFileSelected"
          />
        </div>
      </template>

      <div class="management-layout">
        <aside class="configuration-list" aria-label="配置分类">
          <div class="configuration-list-heading">通用配置</div>
          <button
            type="button"
            class="configuration-list-item"
            :class="{ active: activeSectionId === 'system-settings' }"
            @click="activeSectionId = 'system-settings'"
          >
            <span>系统与外观</span>
            <small>{{ systemDefinitions.length }} 项</small>
          </button>
          <button
            type="button"
            class="configuration-list-item"
            :class="{ active: activeSectionId === 'business-options' }"
            @click="activeSectionId = 'business-options'"
          >
            <span>模具选项</span>
            <small>{{ optionDefinitions.length }} 项</small>
          </button>

          <div class="configuration-list-heading table-heading">页面表格</div>
          <button
            v-for="page in pageTableCatalog"
            :key="page.id"
            type="button"
            class="configuration-list-item"
            :class="{ active: page.id === activeSectionId }"
            @click="activeSectionId = page.id"
          >
            <span>{{ page.label }}</span>
            <small>{{ page.tables.length }} 个表格</small>
          </button>
        </aside>

        <main v-if="activeSectionId === 'system-settings'" class="configuration-settings">
          <div class="page-summary">
            <div>
              <h2>系统与外观</h2>
              <p>统一管理可迁移的系统行为与外观设置。本机文件路径和备份目录仍在“数据与备份”页维护。</p>
            </div>
          </div>

          <section class="option-card tolerance-keyboard-config">
            <div class="option-card-heading">
              <div>
                <h3>公差虚拟键盘</h3>
                <p>配置螺丝规格公差字段的数字、符号与常用词快捷输入。</p>
              </div>
              <el-switch
                v-model="keyboardDraft.enabled"
                inline-prompt
                active-text="开"
                inactive-text="关"
                @change="saveKeyboardSettings"
              />
            </div>
            <el-input-tag
              v-model="keyboardDraft.quickKeys"
              :max="30"
              clearable
              placeholder="输入字符后按 Enter，例如 ±、介厚"
              @change="saveKeyboardSettings"
            />
            <div class="keyboard-preview">
              <span>预览</span>
              <el-tag v-for="key in keyboardDraft.quickKeys" :key="key" effect="plain">{{ key }}</el-tag>
            </div>
            <p class="keyboard-scope">适用于头/垫片大小、头高、长度、牙径、光钉长度；数字键 0–9、00 固定显示。</p>
            <div class="option-actions">
              <el-button type="primary" @click="saveKeyboardSettings">保存</el-button>
              <el-button @click="resetKeyboardSettings">恢复默认</el-button>
            </div>
          </section>

          <div v-loading="loadingSettings" class="option-grid">
            <section v-for="definition in systemDefinitions" :key="definition.id" class="option-card">
              <div class="option-card-heading">
                <div>
                  <h3>{{ definition.label }}</h3>
                  <p>{{ definition.description }}</p>
                </div>
              </div>
              <el-switch
                v-if="definition.editor === 'boolean'"
                v-model="systemValues[definition.id]"
                :disabled="savingSettings[definition.id] || importing"
              />
              <el-input-number
                v-else-if="definition.editor === 'number'"
                v-model="systemValues[definition.id]"
                :min="1"
                :max="100"
                :disabled="savingSettings[definition.id] || importing"
              />
              <el-radio-group
                v-else-if="definition.editor === 'theme'"
                v-model="systemValues[definition.id]"
                :disabled="savingSettings[definition.id] || importing"
              >
                <el-radio-button value="light">浅色</el-radio-button>
                <el-radio-button value="dark">深色</el-radio-button>
                <el-radio-button value="system">跟随系统</el-radio-button>
              </el-radio-group>
              <div class="option-actions">
                <el-button
                  type="primary"
                  :loading="savingSettings[definition.id]"
                  :disabled="importing"
                  @click="saveSystemDefinition(definition)"
                >
                  保存
                </el-button>
                <el-button
                  :disabled="savingSettings[definition.id] || importing"
                  @click="resetSystemDefinition(definition)"
                >
                  恢复默认
                </el-button>
              </div>
            </section>
          </div>
        </main>

        <main v-else-if="activeSectionId === 'business-options'" class="configuration-settings">
          <div class="page-summary">
            <div>
              <h2>模具选项</h2>
              <p>这里维护的候选项会用于对应业务表单，并自动纳入统一配置导入导出。</p>
            </div>
          </div>

          <div v-loading="loadingOptions" class="option-grid">
            <section v-for="definition in optionDefinitions" :key="definition.id" class="option-card">
              <div class="option-card-heading">
                <div>
                  <h3>{{ definition.label }}</h3>
                  <p>{{ definition.description }}</p>
                </div>
                <el-tag size="small" effect="plain">{{ optionValues[definition.id]?.length || 0 }} 项</el-tag>
              </div>
              <el-select
                v-model="optionValues[definition.id]"
                multiple
                filterable
                allow-create
                default-first-option
                :disabled="savingOptions[definition.id] || importing"
                :placeholder="`输入${definition.label.replace('列表', '')}后按回车添加`"
              />
              <div class="option-actions">
                <el-button
                  type="primary"
                  :loading="savingOptions[definition.id]"
                  :disabled="importing"
                  @click="saveOptionDefinition(definition)"
                >
                  保存
                </el-button>
                <el-button
                  :disabled="savingOptions[definition.id] || importing"
                  @click="resetOptionDefinition(definition)"
                >
                  恢复默认
                </el-button>
              </div>
            </section>
          </div>
        </main>

        <main v-else-if="activePage" class="configuration-settings">
          <div class="page-summary">
            <div>
              <h2>{{ activePage.label }}</h2>
              <p>修改后自动保存，并在对应页面即时生效；拖动业务表格表头边界可调整列宽。</p>
            </div>
            <div class="page-summary-actions">
              <el-button type="primary" plain @click="openPage">打开页面</el-button>
              <el-button :loading="resettingAllTables" :disabled="importing" @click="resetAllTables">恢复全部表格默认</el-button>
            </div>
          </div>

          <el-collapse v-model="activeTables" class="table-collapse">
            <el-collapse-item v-for="table in activePage.tables" :key="table.id" :name="table.id">
              <template #title>
                <div class="table-title">
                  <span>{{ table.label }}</span>
                  <el-tag size="small" effect="plain">{{ visibleCount(table.id) }}/{{ table.columns.length }} 列显示</el-tag>
                </div>
              </template>

              <div class="table-toolbar">
                <span>列配置 <small>已拖动的列宽会自动保存</small></span>
                <el-button link type="primary" @click.stop="resetTable(table.id)">恢复默认</el-button>
              </div>

              <el-table :data="table.columns" border stripe class="column-table">
                <el-table-column prop="label" label="列名" min-width="180">
                  <template #default="{ row }">
                    <div class="column-name">
                      <strong>{{ row.label }}</strong>
                      <small>{{ row.id }}<template v-if="columnState(table.id, row.id).width"> · {{ columnState(table.id, row.id).width }}px</template></small>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="显示" width="120" align="center">
                  <template #default="{ row }">
                    <el-switch
                      :model-value="columnState(table.id, row.id).visible"
                      @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'visible', Boolean(value))"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="排序" width="120" align="center">
                  <template #default="{ row }">
                    <el-tooltip :disabled="row.canSort !== false" content="该列不支持排序" placement="top">
                      <span>
                        <el-switch
                          :model-value="columnState(table.id, row.id).sortable"
                          :disabled="row.canSort === false"
                          @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'sortable', Boolean(value))"
                        />
                      </span>
                    </el-tooltip>
                  </template>
                </el-table-column>
                <el-table-column label="筛选" width="120" align="center">
                  <template #default="{ row }">
                    <el-tooltip :disabled="row.canFilter !== false" content="该列不支持筛选" placement="top">
                      <span>
                        <el-switch
                          :model-value="columnState(table.id, row.id).filterable"
                          :disabled="row.canFilter === false"
                          @change="(value: string | number | boolean) => updateColumn(table.id, row.id, 'filterable', Boolean(value))"
                        />
                      </span>
                    </el-tooltip>
                  </template>
                </el-table-column>
              </el-table>
            </el-collapse-item>
          </el-collapse>
        </main>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { Download, Operation, Upload } from '@element-plus/icons-vue'
import { isTauriEnvironment } from '../api'
import { pageTableCatalog } from '../config/tableCatalog'
import {
  applyPortableConfigurationImport,
  createPortableConfigurationExport,
  getPortableConfigurationRegistry,
  parsePortableConfigurationImport,
  type PortableConfigurationDefinition,
  type PortableConfigurationImportPlan,
} from '../config/portableConfigurationRegistry'
import { useTablePreferences, type ColumnPreference } from '../composables/useTablePreferences'
import { useToleranceKeyboard } from '../composables/useToleranceKeyboard'
import { isUserCancellation, showDetailedError } from '../utils/errorFeedback'

const router = useRouter()
const {
  settings: toleranceKeyboardSettings,
  updateToleranceKeyboardSettings,
  resetToleranceKeyboardSettings,
} = useToleranceKeyboard()
const keyboardDraft = ref({
  enabled: toleranceKeyboardSettings.enabled,
  quickKeys: [...toleranceKeyboardSettings.quickKeys],
})

function saveKeyboardSettings() {
  updateToleranceKeyboardSettings({
    enabled: keyboardDraft.value.enabled,
    quickKeys: keyboardDraft.value.quickKeys,
  })
  ElMessage.success('公差虚拟键盘配置已保存')
}

function resetKeyboardSettings() {
  resetToleranceKeyboardSettings()
  keyboardDraft.value = {
    enabled: toleranceKeyboardSettings.enabled,
    quickKeys: [...toleranceKeyboardSettings.quickKeys],
  }
  ElMessage.success('公差虚拟键盘已恢复默认')
}
const registry = getPortableConfigurationRegistry()
const systemDefinitions = registry.filter(definition => ['boolean', 'number', 'theme'].includes(definition.editor))
const optionDefinitions = registry.filter(definition => definition.editor === 'option-list') as PortableConfigurationDefinition<string[]>[]
const activeSectionId = ref('system-settings')
const activeTables = ref<string[]>([])
const exporting = ref(false)
const importing = ref(false)
const resettingAllTables = ref(false)
const loadingSettings = ref(false)
const loadingOptions = ref(false)
const browserFileInput = ref<HTMLInputElement>()
const systemValues = reactive<Record<string, any>>({})
const savingSettings = reactive<Record<string, boolean>>({})
const optionValues = reactive<Record<string, string[]>>({})
const savingOptions = reactive<Record<string, boolean>>({})
const activePage = computed(() => pageTableCatalog.find(page => page.id === activeSectionId.value))
const { getColumnPreference, setColumnPreference, resetTablePreference, resetAllPreferences } = useTablePreferences()

watch(activePage, page => {
  activeTables.value = page?.tables.map(table => table.id) || []
}, { immediate: true })

onMounted(async () => {
  await Promise.all([loadSystemDefinitions(), loadOptionDefinitions()])
})

async function loadSystemDefinitions() {
  loadingSettings.value = true
  try {
    for (const definition of systemDefinitions) {
      systemValues[definition.id] = await definition.load()
    }
  } catch (error) {
    showDetailedError('加载系统配置', error)
  } finally {
    loadingSettings.value = false
  }
}

async function loadOptionDefinitions() {
  loadingOptions.value = true
  try {
    for (const definition of optionDefinitions) {
      optionValues[definition.id] = await definition.load()
    }
  } catch (error) {
    showDetailedError('加载统一配置', error)
  } finally {
    loadingOptions.value = false
  }
}

async function saveSystemDefinition(definition: PortableConfigurationDefinition<any>) {
  if (savingSettings[definition.id] || importing.value) return
  savingSettings[definition.id] = true
  try {
    const normalized = definition.normalize(systemValues[definition.id])
    await definition.save(normalized)
    systemValues[definition.id] = normalized
    ElMessage.success(`${definition.label}已保存`)
  } catch (error) {
    showDetailedError(`保存${definition.label}`, error)
  } finally {
    savingSettings[definition.id] = false
  }
}

async function resetSystemDefinition(definition: PortableConfigurationDefinition<any>) {
  if (definition.defaultValue === undefined || savingSettings[definition.id] || importing.value) return
  savingSettings[definition.id] = true
  try {
    const defaults = definition.normalize(definition.defaultValue)
    await definition.save(defaults)
    systemValues[definition.id] = defaults
    ElMessage.success(`${definition.label}已恢复默认`)
  } catch (error) {
    showDetailedError(`恢复默认${definition.label}`, error)
  } finally {
    savingSettings[definition.id] = false
  }
}

async function saveOptionDefinition(definition: PortableConfigurationDefinition<string[]>) {
  if (savingOptions[definition.id] || importing.value) return
  savingOptions[definition.id] = true
  try {
    const normalized = definition.normalize(optionValues[definition.id] || [])
    await definition.save(normalized)
    optionValues[definition.id] = [...normalized]
    ElMessage.success(`${definition.label}已保存`)
  } catch (error) {
    showDetailedError(`保存${definition.label}`, error)
  } finally {
    savingOptions[definition.id] = false
  }
}

async function resetOptionDefinition(definition: PortableConfigurationDefinition<string[]>) {
  if (!definition.defaultValue || savingOptions[definition.id] || importing.value) return
  savingOptions[definition.id] = true
  try {
    const defaults = definition.normalize(definition.defaultValue)
    await definition.save(defaults)
    optionValues[definition.id] = [...defaults]
    ElMessage.success(`${definition.label}已恢复默认`)
  } catch (error) {
    showDetailedError(`恢复默认${definition.label}`, error)
  } finally {
    savingOptions[definition.id] = false
  }
}

function columnState(tableId: string, columnId: string) {
  return getColumnPreference(tableId, columnId)
}

function updateColumn(tableId: string, columnId: string, key: keyof ColumnPreference, value: boolean) {
  setColumnPreference(tableId, columnId, { [key]: value })
}

function visibleCount(tableId: string) {
  const table = activePage.value?.tables.find(item => item.id === tableId)
  return table?.columns.filter(column => columnState(tableId, column.id).visible).length || 0
}

function resetTable(tableId: string) {
  resetTablePreference(tableId)
  ElMessage.success('已恢复该表格默认配置')
}

async function resetAllTables() {
  if (resettingAllTables.value || importing.value) return
  resettingAllTables.value = true
  try {
    await ElMessageBox.confirm('确定恢复所有页面表格的默认列配置？', '恢复默认', { type: 'warning' })
    resetAllPreferences()
    ElMessage.success('已恢复全部表格默认配置')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('恢复全部表格默认配置', error)
  } finally {
    resettingAllTables.value = false
  }
}

function createExportFilename() {
  const timestamp = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)
  return `mold-configuration-${timestamp}.json`
}

function downloadBrowserFile(filename: string, content: string) {
  const blob = new Blob([content], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

async function handleExport() {
  if (exporting.value || importing.value) return
  exporting.value = true
  try {
    const content = JSON.stringify(await createPortableConfigurationExport(), null, 2)
    const filename = createExportFilename()
    if (isTauriEnvironment()) {
      const filePath = await save({
        defaultPath: filename,
        filters: [{ name: '模具系统配置文件', extensions: ['json'] }],
      })
      if (!filePath) return
      await writeFile(filePath, new TextEncoder().encode(content))
    } else {
      downloadBrowserFile(filename, content)
    }
    ElMessage.success(`全部配置导出成功，共 ${registry.length} 项`)
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导出全部配置', error)
  } finally {
    exporting.value = false
  }
}

async function confirmAndApplyImport(plan: PortableConfigurationImportPlan) {
  const itemText = plan.items.map(item => `• ${item.label}：${item.summary}`).join('\n')
  let applying = false
  await ElMessageBox.confirm(
    `将导入以下全部配置：\n${itemText}\n\n导入会替换当前统一配置，确定继续？`,
    '确认导入全部配置',
    {
      type: 'warning',
      confirmButtonText: '确定导入',
      cancelButtonText: '取消',
      beforeClose: async (action, instance, done) => {
        if (action !== 'confirm') {
          done()
          return
        }
        if (applying) return
        applying = true
        instance.confirmButtonLoading = true
        instance.confirmButtonText = '正在导入...'
        try {
          await applyPortableConfigurationImport(plan)
          await Promise.all([loadSystemDefinitions(), loadOptionDefinitions()])
          done()
          ElMessage.success('统一配置导入成功')
        } catch (error) {
          showDetailedError('导入统一配置', error)
        } finally {
          applying = false
          instance.confirmButtonLoading = false
          instance.confirmButtonText = '确定导入'
        }
      },
    },
  )
}

async function importFromText(content: string) {
  await confirmAndApplyImport(parsePortableConfigurationImport(content))
}

async function handleImport() {
  if (importing.value || exporting.value) return
  if (!isTauriEnvironment()) {
    browserFileInput.value?.click()
    return
  }
  importing.value = true
  try {
    const filePath = await open({
      filters: [{ name: '模具系统配置文件', extensions: ['json'] }],
      multiple: false,
      title: '选择统一配置文件',
    })
    if (!filePath) return
    const bytes = await readFile(filePath as string)
    await importFromText(new TextDecoder().decode(bytes))
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导入统一配置', error, '请确认选择的是由本系统导出的 JSON 配置文件。')
  } finally {
    importing.value = false
  }
}

async function handleBrowserFileSelected(event: Event) {
  if (importing.value || exporting.value) return
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importing.value = true
  try {
    await importFromText(await file.text())
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导入统一配置', error, '请确认选择的是由本系统导出的 JSON 配置文件。')
  } finally {
    importing.value = false
  }
}

function openPage() {
  if (activePage.value) router.push(activePage.value.route)
}
</script>

<style scoped>
.configuration-management {
  height: 100%;
  min-height: 0;
}

.management-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.management-card :deep(.el-card__body) {
  flex: 1;
  min-height: 0;
  padding: 0;
}

.card-header,
.header-actions,
.title-block,
.page-summary,
.page-summary-actions,
.table-title,
.table-toolbar,
.column-name,
.option-card-heading,
.option-actions {
  display: flex;
  align-items: center;
}

.card-header {
  justify-content: space-between;
  gap: 20px;
}

.header-actions,
.page-summary-actions,
.option-actions {
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.header-actions .el-button + .el-button,
.page-summary-actions .el-button + .el-button,
.option-actions .el-button + .el-button {
  margin-left: 0;
}

.browser-file-input {
  display: none;
}

.title-block {
  gap: 12px;
}

.title-block > .el-icon {
  color: var(--el-color-primary);
  font-size: 22px;
}

.title-block > div,
.column-name {
  flex-direction: column;
  align-items: flex-start;
}

.title-block strong {
  color: var(--text-primary);
  font-size: 16px;
}

.title-block span,
.page-summary p,
.column-name small,
.option-card-heading p {
  color: var(--text-secondary);
  font-size: 12px;
}

.management-layout {
  height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
}

.configuration-list {
  min-height: 0;
  padding: 14px 12px;
  overflow-y: auto;
  background: var(--surface-muted);
  border-right: 1px solid var(--border);
}

.configuration-list-heading {
  padding: 4px 12px 8px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 600;
}

.table-heading {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.configuration-list-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 6px;
  padding: 11px 12px;
  color: var(--text-secondary);
  font: inherit;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
}

.configuration-list-item:hover {
  background: var(--surface-hover);
}

.configuration-list-item.active {
  color: var(--el-color-primary);
  font-weight: 600;
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-7);
}

.configuration-list-item small {
  color: var(--text-muted);
  white-space: nowrap;
}

.configuration-settings {
  min-width: 0;
  min-height: 0;
  padding: 20px 24px 32px;
  overflow: auto;
}

.page-summary {
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 18px;
}

.page-summary h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 20px;
}

.page-summary p {
  margin: 5px 0 0;
}

.tolerance-keyboard-config { margin-bottom: 14px; }
.tolerance-keyboard-config :deep(.el-input-tag) { width: 100%; }
.keyboard-preview { display: flex; align-items: center; flex-wrap: wrap; gap: 7px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 9px; background: var(--surface-muted); }
.keyboard-preview > span { margin-right: 3px; color: var(--text-muted); font-size: 12px; }
.keyboard-scope { margin: 0; color: var(--text-secondary); font-size: 12px; line-height: 1.65; }

.option-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
  gap: 14px;
}

.option-card {
  display: grid;
  gap: 14px;
  padding: 18px;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 10px;
}

.option-card-heading {
  justify-content: space-between;
  gap: 16px;
}

.option-card-heading h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 15px;
}

.option-card-heading p {
  margin: 5px 0 0;
  line-height: 1.6;
}

.option-card .el-select {
  width: 100%;
}

.option-actions {
  justify-content: flex-start;
}

.table-collapse {
  border-top: 0;
}

.table-collapse :deep(.el-collapse-item) {
  margin-bottom: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.table-collapse :deep(.el-collapse-item__header) {
  height: 52px;
  padding: 0 16px;
  background: var(--surface-muted);
  border-bottom: 0;
}

.table-collapse :deep(.el-collapse-item__wrap) {
  border-bottom: 0;
}

.table-collapse :deep(.el-collapse-item__content) {
  padding: 14px 16px 16px;
}

.table-title,
.table-toolbar {
  width: 100%;
  justify-content: space-between;
  padding-right: 8px;
}

.table-toolbar {
  margin-bottom: 10px;
  color: var(--text-secondary);
  font-size: 13px;
}

.table-toolbar small {
  margin-left: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

.column-table {
  --el-table-border-color: var(--border);
  --el-table-bg-color: var(--card-bg);
  --el-table-tr-bg-color: var(--card-bg);
  --el-table-header-bg-color: var(--surface-muted);
  --el-table-row-hover-bg-color: var(--surface-hover);
  --el-table-current-row-bg-color: var(--surface-hover);
  --el-table-header-text-color: var(--text-primary);
  --el-table-text-color: var(--text-secondary);
  --el-fill-color-lighter: var(--table-stripe-bg);
  overflow: hidden;
  border-color: var(--border);
  background: var(--card-bg);
}

.column-table :deep(.el-table__inner-wrapper::before) {
  background-color: var(--border);
}

.column-table :deep(.el-switch) {
  --el-switch-on-color: var(--el-color-primary);
  --el-switch-off-color: var(--switch-off-bg);
}

.column-table :deep(.el-switch.is-disabled) {
  --el-switch-on-color: color-mix(in srgb, var(--el-color-primary) 46%, var(--card-bg));
  --el-switch-off-color: var(--switch-disabled-bg);
  opacity: 0.72;
}

.column-table :deep(.el-switch__core) {
  border-color: color-mix(in srgb, currentColor 12%, var(--border));
  transition: background-color 160ms ease, border-color 160ms ease;
}

.column-table :deep(.el-switch:not(.is-disabled):focus-visible .el-switch__core) {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.column-name {
  gap: 2px;
}

.column-name strong {
  color: var(--text-primary);
  font-weight: 500;
}

@media (max-width: 900px) {
  .management-layout {
    grid-template-columns: 170px minmax(0, 1fr);
  }

  .configuration-settings {
    padding: 16px;
  }

  .option-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
