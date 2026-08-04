<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>螺丝规格管理</span>
          <div class="header-right">
            <el-button @click="openPrint('print')">
              <el-icon><Printer /></el-icon>
              打印
            </el-button>
            <el-button type="primary" @click="openPrint('pdf')">
              <el-icon><Download /></el-icon>
              导出 PDF
            </el-button>
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加规格
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

        <DataTable table-id="screw-spec.info" :data="tableData" :loading="loading">
        <ConfigurableTable field="name" title="螺丝名称" width="160" sortable />
        <ConfigurableTable field="headType" title="头型" width="120" sortable :filters="headTypeFilters" :filter-method="exactFilter" />
        <ConfigurableTable field="punch" title="冲头" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.punch" type="primary" :underline="false" @click="showPunchDialog(row)">{{ row.punch }}</el-link>
            <span v-else>-</span>
          </template>
        </ConfigurableTable>
        <ConfigurableTable field="threadType" title="牙型" width="120" sortable :filters="threadTypeFilters" :filter-method="exactFilter" />
        <ConfigurableTable field="die" title="牙板" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.die" type="success" :underline="false" @click="showDieDialog(row)">{{ row.die }}</el-link>
            <span v-else>-</span>
          </template>
        </ConfigurableTable>
        <ConfigurableTable field="headSize" title="头/垫片大小" width="140" sortable />
        <ConfigurableTable field="headHeight" title="头高" width="100" sortable />
        <ConfigurableTable field="length" title="长度" width="100" sortable />
        <ConfigurableTable field="threadDiameter" title="牙径" width="100" sortable />
        <ConfigurableTable field="shankLength" title="光钉长度" width="120" sortable />
        <ConfigurableTable field="wireMaterial" title="线材" width="100" sortable />
        <ConfigurableTable field="plating" title="电镀" width="120" sortable :filters="platingFilters" :filter-method="exactFilter" />
        <ConfigurableTable field="customer" title="客户名" width="120" sortable />
        <ConfigurableTable field="externalId" title="外部ID" width="120" sortable />
        <ConfigurableTable field="remark" title="备注" min-width="140" />
        <ConfigurableTable title="附件" width="100" align="center">
          <template #default="{ row }">
            <el-button class="attachment-count-button" size="small" :type="row._attachmentCount ? 'primary' : 'info'" link @click="openAttachmentPreview(row)">
              <el-icon><Paperclip /></el-icon>
              {{ row._attachmentCount || 0 }}
            </el-button>
          </template>
        </ConfigurableTable>
        <ConfigurableTable title="操作" :width="allowDelete ? 160 : 90" class-name="operation-column" header-class-name="operation-column">
          <template #default="{ row }">
            <el-button size="small" @click="handleEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" v-if="allowDelete" @click="handleDelete(row)">删除</el-button>
          </template>
        </ConfigurableTable>
      </DataTable>
      <div v-if="!loading && tableData.length === 0" class="empty-state">
        <el-empty description="暂无数据" />
      </div>
    </el-card>


    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑螺丝规格' : '添加螺丝规格'" width="800px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="100px">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="螺丝名称" prop="name">
              <SpecInput v-model="form.name" placeholder="如 4.2 X 13" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="螺丝头型">
              <SpecInput v-model="form.headType" placeholder="如 平头" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="冲头" prop="punch">
              <el-select v-model="form.punch" placeholder="请选择冲头" filterable allow-create multiple collapse-tags style="width: 100%">
                <el-option v-for="item in punchOptions" :key="item.name" :label="item.name" :value="item.name">
                  <span>{{ item.name }}</span>
                  <span style="float: right; color: #8492a6; font-size: 12px">{{ item.specs }}</span>
                </el-option>
              </el-select>
              <div class="form-item-spacer"></div>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙型">
              <SpecInput v-model="form.threadType" placeholder="如 自攻" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙板" prop="die">
              <el-select v-model="form.die" placeholder="请选择牙板" filterable allow-create multiple collapse-tags style="width: 100%">
                <el-option v-for="item in dieOptions" :key="item.id" :label="item.label" :value="item.value">
                  <span>{{ item.shortName }}</span>
                  <span v-if="item.specs" style="float: right; color: #8492a6; font-size: 12px">{{ item.specs }}</span>
                </el-option>
              </el-select>
              <div class="form-item-spacer"></div>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="头/垫片大小">
              <SpecInput v-model="form.headSize" placeholder="如 9.1~9.3、5.3±0.1" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="头高">
              <SpecInput v-model="form.headHeight" placeholder="如 2.3~2.4、2.3±0.1" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="长度">
              <SpecInput v-model="form.length" placeholder="如 8±0.5、8-0.5、7.7~8.1" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙径">
              <SpecInput v-model="form.threadDiameter" placeholder="如 2.5~2.6、4.22-0.18" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="光钉长度">
              <SpecInput v-model="form.shankLength" placeholder="如 11、11±0.1" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="线材">
              <SpecInput v-model="form.wireMaterial" placeholder="如 1018" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="电镀">
              <SpecInput v-model="form.plating" placeholder="如 彩锌" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="客户名">
              <SpecInput v-model="form.customer" placeholder="如 客户A" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="外部ID">
              <SpecInput v-model="form.externalId" placeholder="如 A-001" />
            </el-form-item>
          </el-col>
          <el-col :span="24">
            <el-form-item label="备注">
              <el-input v-model="form.remark" type="textarea" />
              <div class="form-item-spacer"></div>
            </el-form-item>
          </el-col>
          <el-col :span="24">
            <el-form-item label="附件">
              <div class="form-attachment-panel">
                <div>
                  <span class="form-attachment-icon"><el-icon><Paperclip /></el-icon></span>
                  <div>
                    <strong>{{ isEdit ? `${editingAttachmentCount} 个附件` : '保存后可添加附件' }}</strong>
                    <span>支持多张图片与 PDF，可添加图形标注</span>
                  </div>
                </div>
                <el-button v-if="isEdit" type="primary" plain @click="openAttachmentEditor(form)">管理与标注</el-button>
                <el-tag v-else type="info" effect="plain">先保存规格</el-tag>
              </div>
            </el-form-item>
          </el-col>
        </el-row>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 冲头关联弹窗 -->
    <RelatedDataDialog
      v-model="punchDialogVisible"
      title="冲头关联"
      description="查看当前螺丝规格关联的冲头与库存状态"
    >
      <ConfigurableVxeTable table-id="screw-spec.punch-dialog" :data="punchDialogItems" border round stripe size="small">
        <ConfigurableTable title="冲头名称" width="120">
          <template #default="{ row: r }">{{ toShortCode(r.name) || r.name }}</template>
        </ConfigurableTable>
        <ConfigurableTable field="spec" title="规格" width="100" />
        <ConfigurableTable field="material" title="材质" width="80" />
        <ConfigurableTable title="当前库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.currentStock ?? '-' }}</template>
        </ConfigurableTable>
        <ConfigurableTable title="安全库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.safetyStock ?? '-' }}</template>
        </ConfigurableTable>
        <ConfigurableTable title="状态" width="90" align="center">
          <template #default="{ row: r }">
            <el-tag v-if="r.status" :type="r.status === '需入库' ? 'danger' : 'success'" size="small" round>{{ r.status }}</el-tag>
            <span v-else>-</span>
          </template>
        </ConfigurableTable>
        <ConfigurableTable title="外显" width="60" align="center">
          <template #default="{ row: item }">
            <el-link :type="matchPunchNames(item.name, punchDialogPrimary) ? 'info' : 'warning'" :underline="false" @click="setPunchPrimary(item)">
              <el-icon><View /></el-icon>
            </el-link>
          </template>
        </ConfigurableTable>
      </ConfigurableVxeTable>
    </RelatedDataDialog>

    <!-- 牙板关联弹窗 -->
    <RelatedDataDialog
      v-model="dieDialogVisible"
      title="牙板关联"
      description="查看当前螺丝规格关联的牙板与库存状态"
    >
      <ConfigurableVxeTable table-id="screw-spec.die-dialog" :data="dieDialogItems" border round stripe size="small">
        <ConfigurableTable field="name" title="牙板名称" width="100" />
        <ConfigurableTable field="machineType" title="机型" width="100" />
        <ConfigurableTable field="wireDiameter" title="线径" width="80" />
        <ConfigurableTable title="当前库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.currentStock ?? '-' }}</template>
        </ConfigurableTable>
        <ConfigurableTable title="安全库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.safetyStock ?? '-' }}</template>
        </ConfigurableTable>
        <ConfigurableTable title="状态" width="90" align="center">
          <template #default="{ row: r }">
            <el-tag v-if="r.status" :type="r.status === '需入库' ? 'danger' : 'success'" size="small" round>{{ r.status }}</el-tag>
            <span v-else>-</span>
          </template>
        </ConfigurableTable>
        <ConfigurableTable title="外显" width="60" align="center">
          <template #default="{ row: item }">
            <el-link :type="item.name === dieDialogPrimary ? 'info' : 'warning'" :underline="false" @click="setDiePrimary(item)">
              <el-icon><View /></el-icon>
            </el-link>
          </template>
        </ConfigurableTable>
      </ConfigurableVxeTable>
    </RelatedDataDialog>

    <ScrewAttachmentWorkspace
      v-model="attachmentWorkspaceVisible"
      :screw-spec-id="attachmentWorkspaceRow.id || ''"
      :screw-name="attachmentWorkspaceRow.name || ''"
      :mode="attachmentWorkspaceMode"
      @changed="handleAttachmentChanged"
    />

    <PrintSettingsDialog
      v-model="printDialogVisible"
      :mode="printAction"
      page-key="screwSpec"
      :columns="screwSpecPrintColumns"
      @confirm="runPrintAction"
    />
    <PrintArea
      page-key="screwSpec"
      :current-page-key="printCurrentPageKey"
      :rows="printRows"
      :title="printTitle"
      :print-time="printTime"
      :settings="printSettings"
      :enabled-columns="printEnabledColumns"
      :paginated-pages="printPaginatedPages"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { View, Paperclip, Printer, Download } from '@element-plus/icons-vue'
import type { FormInstance } from 'element-plus'
import { screwSpecApi, screwAttachmentApi, punchApi, dieApi, punchLinkApi, dieLinkApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { settleNamedRequests, showBatchErrors, showDetailedError } from '../utils/errorFeedback'
import FullscreenToggle from '../components/FullscreenToggle.vue'
import DataTable from '../components/DataTable.vue'
import SpecInput from '../components/SpecInput.vue'
import RelatedDataDialog from '../components/RelatedDataDialog.vue'
import ScrewAttachmentWorkspace from '../components/ScrewAttachmentWorkspace.vue'
import PrintSettingsDialog from '../components/PrintSettingsDialog.vue'
import PrintArea from '../components/PrintArea.vue'
import { screwSpecPrintColumns } from '../config/printColumns'
import { usePrint } from '../composables/usePrint'
import { usePrintSettings } from '../composables/usePrintSettings'
import { toShortCode, matchPunchNames } from '../utils/punchName'
import { parseSpecText } from '../utils/specNormalize'
import { exactFilter, buildFilters } from '../utils/tableFilters'

const { allowDelete } = useAllowDelete()
// 全屏状态由 App 全局提供（isFullscreen 仅用于页面容器样式）
const { isFullscreen } = inject<any>('fullscreen')!

const tableData = ref<any[]>([])
useHighlight(tableData)
const punchList = ref<any[]>([])
const dieList = ref<any[]>([])
const dialogVisible = ref(false)
const isEdit = ref(false)
const loading = ref(true)
const attachmentCounts = ref<Record<string, number>>({})
const attachmentWorkspaceVisible = ref(false)
const attachmentWorkspaceMode = ref<'preview' | 'edit'>('preview')
const attachmentWorkspaceRow = ref<any>({})
const editingAttachmentCount = computed(() => attachmentCounts.value[form.value.id] || 0)

// 冲头关联弹窗
const punchDialogVisible = ref(false)
const punchDialogItems = ref<any[]>([])
const punchDialogPrimary = ref('')
const punchDialogRow = ref<any>({})

// 牙板关联弹窗
const dieDialogVisible = ref(false)
const dieDialogItems = ref<any[]>([])
const dieDialogPrimary = ref('')
const dieDialogRow = ref<any>({})

// 打印与导出
const {
  rows: printRows,
  title: printTitle,
  printTime,
  print,
  exportPdf,
  currentPageKey: printCurrentPageKey,
  paginatedPages: printPaginatedPages,
  enabledColumns: printEnabledColumns,
} = usePrint()
const { settings: printSettings } = usePrintSettings()
const printDialogVisible = ref(false)
const printAction = ref<'print' | 'pdf'>('print')

function openPrint(action: 'print' | 'pdf') {
  if (!tableData.value.length) {
    ElMessage.info('暂无数据可打印')
    return
  }
  printAction.value = action
  printDialogVisible.value = true
}

function runPrintAction() {
  const options = { title: '螺丝规格明细表' }
  if (printAction.value === 'print') {
    print('screwSpec', tableData.value, screwSpecPrintColumns, options)
  } else {
    exportPdf('screwSpec', tableData.value, screwSpecPrintColumns, options)
  }
}

onMounted(() => {
  loadData()
})

const formRef = ref<FormInstance>()
const formRules = { name: [{ required: true, message: '请输入螺丝名称', trigger: 'blur' }] }

// 冲头选项（按名称去重，显示简写+全写）
const punchOptions = computed(() => {
  const names = [...new Set(punchList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => {
    const short = toShortCode(name)
    const display = short || name
    const fullName = short ? name : ''
    return {
      name: display, // 简写作为值
      fullName,       // 全写（如果有）
      label: fullName ? `${display} (${fullName})` : display,
      specs: punchList.value.filter(p => p.name === name).map(p => `${p.spec}${p.material ? '(' + p.material + ')' : ''}`).join('、')
    }
  })
})

// 牙板选项（按名称+机型+线径组合区分）
const dieOptions = computed(() => {
  return dieList.value.filter(d => d.name).map(d => {
    const display = [d.name, d.machineType, d.wireDiameter].filter(Boolean).join(' ')
    const specs = [d.machineType, d.wireDiameter].filter(Boolean).join(' ')
    return { id: d.id, name: display, label: display, value: display, shortName: d.name, specs }
  })
})

const headTypeFilters = computed(() => buildFilters(tableData.value, 'headType'))
const threadTypeFilters = computed(() => buildFilters(tableData.value, 'threadType'))
const platingFilters = computed(() => buildFilters(tableData.value, 'plating'))

const form = ref<any>({
  id: '', customer: '', externalId: '', name: '', headType: '',
  punch: [], threadType: '', die: [], headSize: '', headHeight: '',
  length: '', threadDiameter: '', shankLength: '', wireMaterial: '', plating: '', remark: ''
})

// 解析关联表：保留具体的记录 ID，同时解析出名字用于显示（全写转简写）
function resolveLinks(linkIdField: string, links: any[], infoList: any[]): Record<string, { ids: string[], names: string[] }> {
  const map: Record<string, { ids: string[], names: string[] }> = {}
  for (const link of links) {
    const specId = link.screwSpecId
    const itemId = link[linkIdField]
    if (!specId || !itemId) continue
    if (!map[specId]) map[specId] = { ids: [], names: [] }
    if (!map[specId].ids.includes(itemId)) {
      map[specId].ids.push(itemId)
      const info = infoList.find((i: any) => i.id === itemId)
      const rawName = info ? info.name : itemId
      const shortName = toShortCode(rawName) || rawName
      map[specId].names.push(shortName)
    }
  }
  return map
}

// 根据名字在信息表中查找 ID（多条同名的都返回）
function findIdsByNames(names: string[], infoList: any[], isDie = false): string[] {
  const ids: string[] = []
  const seen = new Set<string>()
  for (const n of names) {
    for (const item of infoList) {
      if (isDie) {
        // 牙板按 名称+机型+线径 组合匹配
        const display = [item.name, item.machineType, item.wireDiameter].filter(Boolean).join(' ')
        if ((display === n || item.name === n) && !seen.has(item.id)) {
          ids.push(item.id)
          seen.add(item.id)
        }
      } else {
        if (matchPunchNames(n, item.name) && !seen.has(item.id)) {
          ids.push(item.id)
          seen.add(item.id)
        }
      }
    }
  }
  return ids
}

async function loadData() {
  loading.value = true
  try {
    const { values, failures } = await settleNamedRequests([
      { label: '螺丝规格信息', request: screwSpecApi.getAll() },
      { label: '冲头信息', request: punchApi.getAll() },
      { label: '牙板信息', request: dieApi.getAll() },
      { label: '冲头关联', request: punchLinkApi.getAll() },
      { label: '牙板关联', request: dieLinkApi.getAll() },
      { label: '附件数量', request: screwAttachmentApi.counts() },
    ])
    const [screws, punches, dies, punchLinks, dieLinks, counts] = values as [
      any[] | undefined,
      any[] | undefined,
      any[] | undefined,
      any[] | undefined,
      any[] | undefined,
      Record<string, number> | undefined,
    ]
    const availablePunches = punches || punchList.value
    const availableDies = dies || dieList.value
    const punchMap = punchLinks ? resolveLinks('punchId', punchLinks, availablePunches) : {}
    const dieMap = dieLinks ? resolveLinks('dieId', dieLinks, availableDies) : {}
    const availableCounts = counts || {}

    if (screws) {
      tableData.value = screws.map((screw: any) => ({
        ...screw,
        _punchIds: punchMap[screw.id]?.ids || [],
        _punchNames: punchMap[screw.id]?.names || [],
        _dieIds: dieMap[screw.id]?.ids || [],
        _dieNames: dieMap[screw.id]?.names || [],
        _attachmentCount: availableCounts[screw.id] || 0,
      }))
    }
    if (counts) attachmentCounts.value = counts
    if (punches) punchList.value = punches
    if (dies) dieList.value = dies
    showBatchErrors('螺丝规格数据加载', failures)
  } finally {
    loading.value = false
  }
}

// ====== 冲头关联弹窗 ======
function showPunchDialog(row: any) {
  punchDialogRow.value = row
  punchDialogPrimary.value = row.punch || ''
  const ids = row._punchIds || []
  const items = ids.map((id: string) => punchList.value.find(p => p.id === id)).filter(Boolean).map((p: any) => ({ ...p }))
  stockCalcApi.calculate('punch').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.punchId === item.id)
      if (match) { item.currentStock = match.currentStock; item.safetyStock = match.safetyStock; item.status = match.status }
    }
    punchDialogItems.value = [...items]
  }).catch(() => { punchDialogItems.value = items })
  punchDialogVisible.value = true
}

async function setPunchPrimary(item: any) {
  if (matchPunchNames(item.name, punchDialogPrimary.value)) return
  try {
    const shortName = toShortCode(item.name) || item.name
    await screwSpecApi.update(punchDialogRow.value.id, { punch: shortName })
    punchDialogVisible.value = false
    loadData()
    ElMessage.success('外显冲头已设置')
  } catch (error) {
    showDetailedError('设置螺丝规格外显冲头', error)
  }
}

// ====== 牙板关联弹窗 ======
function showDieDialog(row: any) {
  dieDialogRow.value = row
  dieDialogPrimary.value = row.die || ''
  const ids = row._dieIds || []
  const items = ids.map((id: string) => dieList.value.find(d => d.id === id)).filter(Boolean).map((d: any) => ({ ...d }))
  stockCalcApi.calculate('die').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.dieId === item.id)
      if (match) { item.currentStock = match.currentStock; item.safetyStock = match.safetyStock; item.status = match.status }
    }
    dieDialogItems.value = [...items]
  }).catch(() => { dieDialogItems.value = items })
  dieDialogVisible.value = true
}

async function setDiePrimary(item: any) {
  if (item.name === dieDialogPrimary.value) return
  try {
    await screwSpecApi.update(dieDialogRow.value.id, { die: item.name })
    dieDialogVisible.value = false
    loadData()
    ElMessage.success('外显牙板已设置')
  } catch (error) {
    showDetailedError('设置螺丝规格外显牙板', error)
  }
}

// ====== 附件 ======
function openAttachmentPreview(row: any) {
  attachmentWorkspaceRow.value = row
  attachmentWorkspaceMode.value = 'preview'
  attachmentWorkspaceVisible.value = true
}

function openAttachmentEditor(row: any) {
  attachmentWorkspaceRow.value = row
  attachmentWorkspaceMode.value = 'edit'
  attachmentWorkspaceVisible.value = true
}

function handleAttachmentChanged(count: number) {
  const id = attachmentWorkspaceRow.value.id
  if (!id) return
  attachmentCounts.value[id] = count
  attachmentWorkspaceRow.value._attachmentCount = count
  const row = tableData.value.find(item => item.id === id)
  if (row) row._attachmentCount = count
}

// ====== CRUD ======
function handleAdd() {
  isEdit.value = false
  form.value = {
    id: '', customer: '', externalId: '', name: '', headType: '',
    punch: [], threadType: '', die: [], headSize: '', headHeight: '',
    length: '', threadDiameter: '', shankLength: '', wireMaterial: '', plating: '', remark: ''
  }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = {
    ...row,
    punch: [...(row._punchNames || [])],
    die: [...(row._dieNames || [])]
  }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此规格？', '提示', { type: 'warning' })
    await screwSpecApi.remove(row.id)
    // 清理关联表
    const [pl, dl] = await Promise.all([punchLinkApi.getAll(), dieLinkApi.getAll()])
    for (const l of pl) { if (l.screwSpecId === row.id) await punchLinkApi.remove(l.id) }
    for (const l of dl) { if (l.screwSpecId === row.id) await dieLinkApi.remove(l.id) }
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') showDetailedError('删除螺丝规格及关联', error)
  }
}

// 同步关联表：先删旧的，再建新的
async function syncLinks(screwSpecId: string, nameField: string, names: string[], linkApi: any, infoList: any[], isDie = false) {
  const allLinks = await linkApi.getAll()
  for (const l of allLinks.filter((l: any) => l.screwSpecId === screwSpecId)) {
    await linkApi.remove(l.id)
  }
  const ids = findIdsByNames(names, infoList, isDie)
  for (const id of ids) {
    await linkApi.add({ [nameField]: id, screwSpecId })
  }
}

// ====== 重复规格判断（命中任一规则即提示，用户可强制保存）======
// 规则 A：名称 + 头型 + 牙型 三个字段全部一致（至少一个非空，避免空记录误报）
// 规则 B：外部ID + 客户 全部一致且均非空（留空不算）
function specFieldKey(value: unknown): string {
  if (value === undefined || value === null) return ''
  return parseSpecText(String(value)).key
}

function duplicateReason(a: Record<string, any>, b: Record<string, any>): string {
  const nameA = specFieldKey(a.name)
  const nameB = specFieldKey(b.name)
  const headA = specFieldKey(a.headType)
  const headB = specFieldKey(b.headType)
  const threadA = specFieldKey(a.threadType)
  const threadB = specFieldKey(b.threadType)
  if ((nameA || headA || threadA) && nameA === nameB && headA === headB && threadA === threadB) {
    return `名称、头型、牙型相同：${b.name || '-'} / ${b.headType || '-'} / ${b.threadType || '-'}`
  }

  const extA = specFieldKey(a.externalId)
  const extB = specFieldKey(b.externalId)
  const customerA = specFieldKey(a.customer)
  const customerB = specFieldKey(b.customer)
  if (extA && extB && customerA && customerB && extA === extB && customerA === customerB) {
    return `外部ID、客户相同：${b.externalId} / ${b.customer}`
  }

  return ''
}

function buildDuplicateReport(): string[] {
  const lines: string[] = []
  const editingId = isEdit.value ? form.value.id : ''
  for (const row of tableData.value) {
    if (row.id === editingId) continue
    const reason = duplicateReason(form.value, row)
    if (reason) lines.push(`• ${reason}`)
  }
  return lines
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      // 重复规格确认：程序只提示，由用户决定是否新增（可强制保存）
      const duplicateReport = buildDuplicateReport()
      if (duplicateReport.length) {
        try {
          await ElMessageBox.confirm(
            `检测到与现有记录相同：\n\n${duplicateReport.join('\n')}\n\n` +
            '如果这是不同客户或不同要求的独立规格，请点「仍要保存」继续；\n' +
            '如果重复了，请点「返回修改」。',
            '重复规格确认',
            {
              type: 'warning',
              confirmButtonText: '仍要保存',
              cancelButtonText: '返回修改',
              distinguishCancelAndClose: true,
            },
          )
        } catch {
          return // 用户选择返回修改或直接关闭
        }
      }

      const punchNames = Array.isArray(form.value.punch) ? form.value.punch : []
      const dieNames = Array.isArray(form.value.die) ? form.value.die : []
      // 主表只存外显的第一个名字
      const payload = {
        ...form.value,
        punch: punchNames[0] || '',
        die: dieNames[0] || ''
      }
      let specId: string
      if (isEdit.value) {
        await screwSpecApi.update(form.value.id, payload)
        specId = form.value.id
      } else {
        const result = await screwSpecApi.add(payload)
        specId = result.id
      }
      // 同步关联表（用信息表的 ID）
      await syncLinks(specId, 'punchId', punchNames, punchLinkApi, punchList.value)
      await syncLinks(specId, 'dieId', dieNames, dieLinkApi, dieList.value, true)
      dialogVisible.value = false
      loadData()
      ElMessage.success(isEdit.value ? '规格已更新' : '规格已添加')
    } catch (error) {
      showDetailedError(isEdit.value ? '更新螺丝规格及关联' : '添加螺丝规格及关联', error)
    }
  })
}
</script>

<style scoped>
.page-container.is-fullscreen { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2000; background: #fff; padding: 0; overflow: visible; }
.page-container.is-fullscreen .el-card { height: 100%; display: flex; flex-direction: column; margin: 0; border: none; border-radius: 0; box-shadow: none; }
.page-container.is-fullscreen .el-card__header { display: none; }
.page-container.is-fullscreen .el-card__body { flex: 1; overflow: hidden; padding: 12px; }

/* 编辑对话框：所有字段统一高度（输入框 + 22px 信息行），无论有误预览/警告/普通输入都整齐 */
.form-item-spacer { height: 22px; }

.header-right { display: flex; gap: 8px; margin-left: auto; }
.attachment-count-button { min-width: 46px; font-weight: 600; }
.form-attachment-panel { width: 100%; min-height: 72px; padding: 12px 14px; display: flex; align-items: center; justify-content: space-between; gap: 12px; border: 1px dashed var(--border-strong); border-radius: 12px; background: var(--surface-muted); }
.form-attachment-panel > div { min-width: 0; display: flex; align-items: center; gap: 11px; }
.form-attachment-panel > div > div { min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.form-attachment-panel strong { color: var(--text-primary); font-size: 13px; }
.form-attachment-panel span { color: var(--text-muted); font-size: 11px; }
.form-attachment-icon { width: 38px; height: 38px; display: grid; place-items: center; flex-shrink: 0; border-radius: 10px; color: var(--primary); background: color-mix(in srgb, var(--primary) 12%, var(--card-bg)); }
.form-attachment-icon .el-icon { font-size: 18px; }
</style>