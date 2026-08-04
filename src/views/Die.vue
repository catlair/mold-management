<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Grid /></el-icon>
          <span>牙板管理</span>
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
              添加牙板
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="牙板信息" name="info">
           <DataTable table-id="die.info" :data="dieList" :loading="loading">
            <ConfigurableTable field="name" title="名称" width="160" sortable>
              <template #default="{ row }">
                <el-link type="primary" :underline="false" @click="showLinkedScrews(row)">{{ row.name }}</el-link>
              </template>
            </ConfigurableTable>
            <ConfigurableTable field="machineType" title="机型" width="120" sortable :filters="machineTypeFilters" :filter-method="filterHandler" />
            <ConfigurableTable field="wireDiameter" title="线径" width="100" sortable />
            <ConfigurableTable field="safetyStock" title="安全库存" width="100" sortable />
            <ConfigurableTable field="currentStock" title="当前库存" width="100" sortable />
            <ConfigurableTable field="status" title="库存状态" width="100" sortable>
              <template #default="{ row }">
                <el-tag v-if="row.status" :type="row.status === '需入库' ? 'danger' : 'success'" effect="dark" round size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </ConfigurableTable>
            <ConfigurableTable field="remark" title="备注" min-width="120" />
            <ConfigurableTable title="操作" width="150" class-name="operation-column" header-class-name="operation-column">
              <template #default="{ row }">
                <el-button size="small" @click="handleEdit(row)">编辑</el-button>
                <el-button size="small" type="danger" v-if="allowDelete" @click="handleDelete(row)">删除</el-button>
              </template>
            </ConfigurableTable>
          </DataTable>
        </el-tab-pane>

        <el-tab-pane label="入库记录" name="order" class="record-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">入库记录</div>
              <div class="record-heading__count">共 {{ orderList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增入库</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="die.order" class="record-table" :data="orderPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="牙板名称" width="32%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getDieName(row.dieId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="quantity" title="入库数量" width="16%" min-width="120" sortable />
              <ConfigurableTable field="orderDate" title="入库时间" width="24%" min-width="180" sortable />
              <ConfigurableTable field="remark" title="备注" width="28%" min-width="180" />
            </ConfigurableVxeTable>
          </div>
          <div class="record-pagination-bar">
            <el-pagination
              v-model:current-page="orderCurrentPage"
              v-model:page-size="orderPageSize"
              :page-sizes="[10, 20, 50]"
              :total="orderList.length"
              layout="total, sizes, prev, pager, next"
              small
              class="record-pagination"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane label="领用记录" name="use" class="record-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">领用记录</div>
              <div class="record-heading__count">共 {{ useList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showUseDialog = true">新增领用</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="die.use" class="record-table" :data="usePaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="牙板名称" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getDieName(row.dieId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="user" title="领用人" width="16%" min-width="140" sortable />
              <ConfigurableTable field="quantity" title="领用数量" width="14%" min-width="120" sortable />
              <ConfigurableTable field="useDate" title="领用时间" width="22%" min-width="180" sortable />
              <ConfigurableTable field="remark" title="备注" width="22%" min-width="180" />
            </ConfigurableVxeTable>
          </div>
          <div class="record-pagination-bar">
            <el-pagination
              v-model:current-page="useCurrentPage"
              v-model:page-size="usePageSize"
              :page-sizes="[10, 20, 50]"
              :total="useList.length"
              layout="total, sizes, prev, pager, next"
              small
              class="record-pagination"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane label="螺丝规格关联" name="link" class="association-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">螺丝规格关联</div>
              <div class="record-heading__count">共 {{ linkList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="die.link" class="record-table" :data="linkPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="牙板" width="28%" min-width="220">
                <template #default="{ row }">
                  {{ getDieName(row.dieId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable title="螺丝规格" width="30%" min-width="240">
                <template #default="{ row }">
                  {{ getScrewSpecName(row.screwSpecId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="remark" title="备注" width="27%" min-width="200" />
              <ConfigurableTable title="操作" width="15%" min-width="120" class-name="operation-column" header-class-name="operation-column">
                <template #default="{ row }">
                  <el-button size="small" type="danger" v-if="allowDelete" @click="handleDeleteLink(row)">删除</el-button>
                </template>
              </ConfigurableTable>
            </ConfigurableVxeTable>
          </div>
          <div class="record-pagination-bar">
            <el-pagination
              v-model:current-page="linkCurrentPage"
              v-model:page-size="linkPageSize"
              :page-sizes="[10, 20, 50]"
              :total="linkList.length"
              layout="total, sizes, prev, pager, next"
              small
              class="record-pagination"
            />
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑牙板' : '添加牙板'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="机型" prop="machineType">
          <el-select
            v-model="form.machineType"
            filterable
            allow-create
            default-first-option
            placeholder="请选择或输入机型"
            style="width: 100%"
          >
            <el-option v-for="item in machineTypeOptions" :key="item" :label="item" :value="item" />
          </el-select>
        </el-form-item>
        <el-form-item label="线径" prop="wireDiameter">
          <el-input v-model="form.wireDiameter" />
        </el-form-item>
        <el-form-item label="安全库存">
          <el-input-number v-model="form.safetyStock" :min="0" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" :rows="2" />
        </el-form-item>
        <el-alert
          v-if="duplicateMatch"
          class="duplicate-alert"
          :type="duplicateMatch.kind === 'exact' ? 'error' : 'warning'"
          :closable="false"
          show-icon
          :title="duplicateMatch.kind === 'exact' ? '已存在完全相同的牙板' : '发现名称相近的牙板，请核对'"
        >
          <template #default>
            <div class="duplicate-record">
              <strong>{{ getDieName(duplicateMatch.record.id) }}</strong>
              <span v-if="duplicateMatch.kind === 'similar' && duplicateMatch.differingFields.length">
                差异字段：{{ duplicateMatch.differingFields.join('、') }}
              </span>
            </div>
          </template>
        </el-alert>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" :disabled="duplicateMatch?.kind === 'exact'" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 入库对话框 -->
    <el-dialog v-model="showOrderDialog" title="新增入库" width="500px">
      <el-form ref="orderFormRef" :model="orderForm" :rules="orderFormRules" label-width="80px">
        <el-form-item label="牙板" prop="dieId">
          <el-select v-model="orderForm.dieId" placeholder="请选择牙板" filterable>
            <el-option v-for="item in dieList" :key="item.id" :label="`${item.name} (${item.machineType}, ${item.wireDiameter})`" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number v-model="orderForm.quantity" :min="1" />
        </el-form-item>
        <el-form-item label="入库时间">
          <el-date-picker v-model="orderForm.orderDate" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="orderForm.remark" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showOrderDialog = false">取消</el-button>
        <el-button type="primary" @click="handleOrderSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 领用对话框 -->
    <el-dialog v-model="showUseDialog" title="新增领用" width="500px">
      <el-form ref="useFormRef" :model="useForm" :rules="useFormRules" label-width="80px">
        <el-form-item label="牙板" prop="dieId">
          <el-select v-model="useForm.dieId" placeholder="请选择牙板" filterable>
            <el-option v-for="item in dieList" :key="item.id" :label="`${item.name} (${item.machineType}, ${item.wireDiameter})`" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="领用人">
          <el-input v-model="useForm.user" />
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number v-model="useForm.quantity" :min="1" />
        </el-form-item>
        <el-form-item label="领用时间">
          <el-date-picker v-model="useForm.useDate" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="useForm.remark" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showUseDialog = false">取消</el-button>
        <el-button type="primary" @click="handleUseSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 关联对话框 -->
    <el-dialog v-model="showLinkDialog" title="新增螺丝规格关联" width="500px">
      <el-form ref="linkFormRef" :model="linkForm" :rules="linkFormRules" label-width="100px">
        <el-form-item label="牙板">
          <el-select v-model="linkForm.dieId" placeholder="请选择牙板" filterable>
            <el-option
              v-for="item in dieList"
              :key="item.id"
              :label="`${item.name} (${item.machineType}${item.wireDiameter ? ' - ' + item.wireDiameter : ''})`"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="螺丝规格">
          <el-select v-model="linkForm.screwSpecId" placeholder="请选择螺丝规格" filterable>
            <el-option
              v-for="item in screwSpecList"
              :key="item.id"
              :label="`${item.name}${item.customer ? ' (' + item.customer + ')' : ''}`"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="linkForm.remark" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showLinkDialog = false">取消</el-button>
        <el-button type="primary" @click="handleLinkSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 关联螺丝对话框 -->
    <RelatedDataDialog
      v-model="showLinkedScrewsDialog"
      :title="`关联螺丝 · ${linkedDieName}`"
      description="查看当前牙板适用的螺丝规格"
    >
      <ConfigurableVxeTable table-id="die.linked-screws" :data="linkedScrews" border round stripe style="width: 100%" :loading="linkedLoading">
        <ConfigurableTable field="name" title="螺丝名称" width="150" sortable />
        <ConfigurableTable field="headType" title="头型" width="100" />
        <ConfigurableTable field="threadType" title="牙型" width="100" />
        <ConfigurableTable field="headSize" title="头/垫片大小" width="120" />
        <ConfigurableTable field="headHeight" title="头高" width="80" />
        <ConfigurableTable field="length" title="长度" width="80" />
        <ConfigurableTable field="threadDiameter" title="牙径" width="80" />
        <ConfigurableTable field="wireMaterial" title="线材" width="80" />
        <ConfigurableTable field="remark" title="备注" min-width="120" />
      </ConfigurableVxeTable>
      <div v-if="!linkedLoading && linkedScrews.length === 0" class="related-dialog-empty">
        该牙板暂无关联螺丝规格
      </div>
    </RelatedDataDialog>

    <PrintSettingsDialog
      v-model="printDialogVisible"
      :mode="printAction"
      page-key="die"
      :columns="diePrintColumns"
      @confirm="runPrintAction"
    />
    <PrintArea
      page-key="die"
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
import type { FormInstance } from 'element-plus'
import { Grid, Plus, Printer, Download } from '@element-plus/icons-vue'
import { dieApi, dieOrderApi, dieUseApi, dieLinkApi, screwSpecApi, stockCalcApi, dieMachineTypeApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { settleNamedRequests, showBatchErrors, showDetailedError } from '../utils/errorFeedback'
import DataTable from '../components/DataTable.vue'
import RelatedDataDialog from '../components/RelatedDataDialog.vue'
import FullscreenToggle from '../components/FullscreenToggle.vue'
import PrintSettingsDialog from '../components/PrintSettingsDialog.vue'
import PrintArea from '../components/PrintArea.vue'
import { diePrintColumns } from '../config/printColumns'
import { usePrint } from '../composables/usePrint'
import { usePrintSettings } from '../composables/usePrintSettings'
import {
  dieUniqueKey,
  duplicateErrorMessage,
  findDieDuplicate,
  isDuplicateError,
} from '../utils/duplicateDetection'

const { allowDelete } = useAllowDelete()
// 全屏状态由 App 全局提供（isFullscreen 仅用于页面容器样式）
const { isFullscreen } = inject<any>('fullscreen')!

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
  if (!dieList.value.length) {
    ElMessage.info('暂无数据可打印')
    return
  }
  printAction.value = action
  printDialogVisible.value = true
}

function runPrintAction() {
  const options = { title: '牙板信息明细表' }
  if (printAction.value === 'print') {
    print('die', dieList.value, diePrintColumns, options)
  } else {
    exportPdf('die', dieList.value, diePrintColumns, options)
  }
}

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const activeTab = ref('info')
const dieList = ref<any[]>([])
useHighlight(dieList)
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const linkList = ref<any[]>([])
const screwSpecList = ref<any[]>([])

const showLinkedScrewsDialog = ref(false)
const linkedScrews = ref<any[]>([])
const linkedDieName = ref('')
const linkedLoading = ref(false)

async function showLinkedScrews(die: any) {
  linkedDieName.value = die.name
  linkedScrews.value = []
  linkedLoading.value = true
  showLinkedScrewsDialog.value = true
  try {
    const links = linkList.value.filter(l => l.dieId === die.id)
    const screwIds = links.map(l => l.screwSpecId)
    linkedScrews.value = screwSpecList.value.filter(s => screwIds.includes(s.id))
  } catch (error) {
    showDetailedError('加载牙板关联螺丝', error)
  } finally {
    linkedLoading.value = false
  }
}
const loading = ref(true)

const orderCurrentPage = ref(1)
const orderPageSize = ref(10)
const orderPaginated = computed(() => {
  const start = (orderCurrentPage.value - 1) * orderPageSize.value
  return orderList.value.slice(start, start + orderPageSize.value)
})
const useCurrentPage = ref(1)
const usePageSize = ref(10)
const usePaginated = computed(() => {
  const start = (useCurrentPage.value - 1) * usePageSize.value
  return useList.value.slice(start, start + usePageSize.value)
})
const linkCurrentPage = ref(1)
const linkPageSize = ref(10)
const linkPaginated = computed(() => {
  const start = (linkCurrentPage.value - 1) * linkPageSize.value
  return linkList.value.slice(start, start + linkPageSize.value)
})

const machineTypeOptions = ref<string[]>(['003', '3/16', '1/4', '6R'])

// 筛选选项
const machineTypeFilters = computed(() => {
  const types = [...new Set(dieList.value.map(item => item.machineType).filter(Boolean))]
  return types.map(t => ({ label: t, value: t }))
})

function filterHandler({ value, row, column }: any) {
  const property = column.property
  return row[property] === value
}

// 获取牙板完整标识
function getDieName(dieId: string) {
  const die = dieList.value.find(d => d.id === dieId)
  return die ? `${die.name} (${die.machineType}, ${die.wireDiameter})` : dieId
}

// 获取螺丝规格名称
function getScrewSpecName(screwSpecId: string) {
  const spec = screwSpecList.value.find(s => s.id === screwSpecId)
  return spec ? spec.name : screwSpecId
}

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const originalUniqueKey = ref('')
const form = ref({ id: '', name: '', machineType: '', wireDiameter: '', safetyStock: 0, remark: '' })
const duplicateMatch = computed(() => {
  if (!form.value.name.trim() || !form.value.machineType.trim() || !form.value.wireDiameter.trim()) return null
  if (isEdit.value && dieUniqueKey(form.value) === originalUniqueKey.value) return null
  return findDieDuplicate(
    form.value,
    dieList.value,
    isEdit.value ? form.value.id : '',
  )
})

const showOrderDialog = ref(false)
const orderForm = ref({ dieId: '', quantity: 1, orderDate: getCurrentDateTime(), remark: '' })

const showUseDialog = ref(false)
const useForm = ref({ dieId: '', user: '', quantity: 1, useDate: getCurrentDateTime(), remark: '' })

const showLinkDialog = ref(false)
const linkForm = ref({ dieId: '', screwSpecId: '', remark: '' })

// 表单引用
const formRef = ref<FormInstance>()
const orderFormRef = ref<FormInstance>()
const useFormRef = ref<FormInstance>()
const linkFormRef = ref<FormInstance>()

// 验证规则
const formRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  machineType: [{ required: true, message: '请输入机型', trigger: 'blur' }],
  wireDiameter: [{ required: true, message: '请输入线径', trigger: 'blur' }],
}
const orderFormRules = {
  dieId: [{ required: true, message: '请选择牙板', trigger: 'change' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const useFormRules = {
  dieId: [{ required: true, message: '请选择牙板', trigger: 'change' }],
  user: [{ required: true, message: '请输入领用人', trigger: 'blur' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const linkFormRules = {
  dieId: [{ required: true, message: '请选择牙板', trigger: 'change' }],
  screwSpecId: [{ required: true, message: '请选择螺丝规格', trigger: 'change' }],
}

onMounted(() => {
  loadData()
  loadMachineTypeOptions()
})

async function loadMachineTypeOptions() {
  try {
    const options = await dieMachineTypeApi.get()
    if (options.length > 0) machineTypeOptions.value = options
  } catch (error) {
    showDetailedError('加载牙板机型列表', error)
  }
}

async function loadData() {
  loading.value = true
  try {
    const { values, failures } = await settleNamedRequests([
      { label: '牙板信息', request: dieApi.getAll() },
      { label: '入库记录', request: dieOrderApi.getAll() },
      { label: '领用记录', request: dieUseApi.getAll() },
      { label: '螺丝规格关联', request: dieLinkApi.getAll() },
      { label: '螺丝规格信息', request: screwSpecApi.getAll() },
      { label: '库存计算', request: stockCalcApi.calculate('die') },
    ])
    const [dies, orders, uses, links, screwSpecs, stockData] = values as Array<any[] | undefined>

    if (dies) {
      const stockMap: Record<string, any> = {}
      stockData?.forEach((item: any) => { stockMap[item.dieId] = item })
      dieList.value = dies.map((die: any) => ({
        ...die,
        currentStock: stockData ? (stockMap[die.id]?.currentStock ?? '') : '',
        safetyStock: stockMap[die.id]?.safetyStock ?? die.safetyStock,
        status: stockData ? (stockMap[die.id]?.status ?? '') : '',
      }))
    }
    if (orders) {
      orderList.value = orders
      orderCurrentPage.value = 1
    }
    if (uses) {
      useList.value = uses
      useCurrentPage.value = 1
    }
    if (links) {
      linkList.value = links
      linkCurrentPage.value = 1
    }
    if (screwSpecs) screwSpecList.value = screwSpecs
    showBatchErrors('牙板数据加载', failures)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  originalUniqueKey.value = ''
  form.value = { id: '', name: '', machineType: '', wireDiameter: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  originalUniqueKey.value = dieUniqueKey(row)
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此牙板？', '提示', { type: 'warning' })
    await dieApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除牙板', error)
    }
  }
}

async function handleSubmit() {
  if (!formRef.value || submitting.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    if (duplicateMatch.value?.kind === 'exact') {
      ElMessage.error('已存在完全相同的牙板，请勿重复保存')
      return
    }

    submitting.value = true
    try {
      if (isEdit.value) {
        await dieApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await dieApi.add(form.value)
        ElMessage.success('添加成功')
      }
      dialogVisible.value = false
      await loadData()
    } catch (error) {
      showDetailedError(
        isEdit.value ? '更新牙板' : '添加牙板',
        error,
        isDuplicateError(error) ? duplicateErrorMessage(error) : undefined,
      )
    } finally {
      submitting.value = false
    }
  })
}

async function handleOrderSubmit() {
  if (!orderFormRef.value) return
  await orderFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await dieOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { dieId: '', quantity: 1, orderDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加牙板入库记录', error)
    }
  })
}

async function handleUseSubmit() {
  if (!useFormRef.value) return
  await useFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await dieUseApi.add(useForm.value)
      ElMessage.success('领用记录添加成功')
      showUseDialog.value = false
      useForm.value = { dieId: '', user: '', quantity: 1, useDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加牙板领用记录', error)
    }
  })
}

async function handleLinkSubmit() {
  if (!linkFormRef.value) return
  await linkFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await dieLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { dieId: '', screwSpecId: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加牙板螺丝规格关联', error)
    }
  })
}

async function handleDeleteLink(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此关联？', '提示', { type: 'warning' })
    await dieLinkApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除牙板螺丝规格关联', error)
    }
  }
}
</script>

<style scoped>
.stock-center {
  display: flex;
  justify-content: center;
}
.stock-center :deep(.el-table) {
  width: auto !important;
}
.page-container.is-fullscreen { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2000; background: #fff; padding: 0; overflow: auto; }
.page-container.is-fullscreen .el-card { height: 100%; display: flex; flex-direction: column; margin: 0; border: none; border-radius: 0; box-shadow: none; }
.page-container.is-fullscreen .el-card__header { display: none; }
.page-container.is-fullscreen .el-card__body { flex: 1; overflow: hidden; padding: 12px; }
.page-container.is-fullscreen .el-table__body-wrapper { overflow: auto !important; }
.page-container.is-fullscreen .el-table .el-table__fixed { height: calc(100% - 14px) !important; }

.header-right {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.duplicate-alert {
  margin-top: 4px;
}

.duplicate-record {
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow-wrap: anywhere;
}
</style>