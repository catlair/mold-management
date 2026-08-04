<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><SetUp /></el-icon>
          <span>冲头管理</span>
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
              添加冲头
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="冲头信息" name="info">
           <DataTable table-id="punch.info" :data="punchList" :loading="loading">
            <ConfigurableTable field="name" title="名称" width="160" sortable>
              <template #default="{ row }">
                <el-link type="primary" :underline="false" @click="showLinkedScrews(row)">{{ row.name }}</el-link>
              </template>
            </ConfigurableTable>
            <ConfigurableTable field="spec" title="规格" width="80" sortable />
            <ConfigurableTable field="material" title="材质" width="120" sortable :filters="materialFilters" :filter-method="exactFilter" />
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
            <ConfigurableVxeTable table-id="punch.order" class="record-table" :data="orderPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="冲头" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getPunchName(row.punchId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="quantity" title="入库数量" width="14%" min-width="120" sortable />
              <ConfigurableTable field="orderDate" title="入库时间" width="22%" min-width="180" sortable />
              <ConfigurableTable field="status" title="到货状态" width="16%" min-width="140" sortable :filters="STATUS_FILTERS" :filter-method="exactFilter" />
              <ConfigurableTable field="remark" title="备注" width="22%" min-width="180" />
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
            <ConfigurableVxeTable table-id="punch.use" class="record-table" :data="usePaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="冲头" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getPunchName(row.punchId) }}
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
            <ConfigurableVxeTable table-id="punch.link" class="record-table" :data="linkPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="冲头" width="28%" min-width="220">
                <template #default="{ row }">
                  {{ getPunchName(row.punchId) }}
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

    <!-- 关联螺丝对话框 -->
    <RelatedDataDialog
      v-model="showLinkedScrewsDialog"
      :title="`关联螺丝 · ${linkedPunchName}`"
      description="查看当前冲头适用的螺丝规格"
    >
      <ConfigurableVxeTable table-id="punch.linked-screws" :data="linkedScrews" border round stripe style="width: 100%" :loading="linkedLoading">
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
        该冲头暂无关联螺丝规格
      </div>
    </RelatedDataDialog>

    <PrintSettingsDialog
      v-model="printDialogVisible"
      :mode="printAction"
      page-key="punch"
      :columns="punchPrintColumns"
      @confirm="runPrintAction"
    />
    <PrintArea
      page-key="punch"
      :current-page-key="printCurrentPageKey"
      :rows="printRows"
      :title="printTitle"
      :print-time="printTime"
      :settings="printSettings"
      :enabled-columns="printEnabledColumns"
      :paginated-pages="printPaginatedPages"
    />

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑冲头' : '添加冲头'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="规格" prop="spec">
          <el-select
            v-model="form.spec"
            filterable
            allow-create
            default-first-option
            placeholder="请选择或输入规格"
            style="width: 100%"
          >
            <el-option v-for="item in specOptions" :key="item" :label="item" :value="item" />
          </el-select>
        </el-form-item>
        <el-form-item label="材质">
          <el-input v-model="form.material" />
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
          :title="duplicateMatch.kind === 'exact' ? '已存在完全相同的冲头' : '发现名称相近的冲头，请核对'"
        >
          <template #default>
            <div class="duplicate-record">
              <strong>{{ getPunchName(duplicateMatch.record.id) }}</strong>
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
        <el-form-item label="冲头" prop="punchId">
          <el-select v-model="orderForm.punchId" placeholder="请选择冲头" filterable>
            <el-option v-for="item in punchList" :key="item.id" :label="`${item.name} ${item.spec}${item.material ? ' ' + item.material : ''}`" :value="item.id" />
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
        <el-form-item label="冲头" prop="punchId">
          <el-select v-model="useForm.punchId" placeholder="请选择冲头" filterable>
            <el-option v-for="item in punchList" :key="item.id" :label="`${item.name} ${item.spec}${item.material ? ' ' + item.material : ''}`" :value="item.id" />
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
        <el-form-item label="冲头">
          <el-select v-model="linkForm.punchId" placeholder="请选择冲头" filterable>
            <el-option
              v-for="item in punchList"
              :key="item.id"
              :label="`${item.name} ${item.spec}${item.material ? ' ' + item.material : ''}`"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { SetUp, Plus, Printer, Download } from '@element-plus/icons-vue'
import { punchApi, punchOrderApi, punchUseApi, punchLinkApi, screwSpecApi, stockCalcApi, punchSpecApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { settleNamedRequests, showBatchErrors, showDetailedError } from '../utils/errorFeedback'
import DataTable from '../components/DataTable.vue'
import RelatedDataDialog from '../components/RelatedDataDialog.vue'
import FullscreenToggle from '../components/FullscreenToggle.vue'
import PrintSettingsDialog from '../components/PrintSettingsDialog.vue'
import PrintArea from '../components/PrintArea.vue'
import { punchPrintColumns } from '../config/printColumns'
import { usePrint } from '../composables/usePrint'
import { usePrintSettings } from '../composables/usePrintSettings'
import { exactFilter, STATUS_FILTERS, buildFilters } from '../utils/tableFilters'
import { toFullName } from '../utils/punchName'
import {
  duplicateErrorMessage,
  findPunchDuplicate,
  isDuplicateError,
  punchUniqueKey,
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
  if (!punchList.value.length) {
    ElMessage.info('暂无数据可打印')
    return
  }
  printAction.value = action
  printDialogVisible.value = true
}

function runPrintAction() {
  const options = { title: '冲头信息明细表' }
  if (printAction.value === 'print') {
    print('punch', punchList.value, punchPrintColumns, options)
  } else {
    exportPdf('punch', punchList.value, punchPrintColumns, options)
  }
}

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const activeTab = ref('info')
const punchList = ref<any[]>([])
useHighlight(punchList)
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const linkList = ref<any[]>([])
const screwSpecList = ref<any[]>([])
const loading = ref(true)
const specOptions = ref<string[]>(['12*15', '14*15', '18*18'])

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

// 筛选选项
const materialFilters = computed(() => buildFilters(punchList.value, 'material'))

// 获取冲头完整标识
function getPunchName(punchId: string) {
  const punch = punchList.value.find(p => p.id === punchId)
  if (!punch) return punchId
  const parts = [punch.name, punch.spec]
  if (punch.material) parts.push(punch.material)
  return parts.join(' ')
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
const form = ref({ id: '', name: '', spec: '', material: '', safetyStock: 0, remark: '' })
const normalizedPunchForm = computed(() => {
  const fullName = toFullName(form.value.name)
  return { ...form.value, name: fullName || form.value.name }
})
const duplicateMatch = computed(() => {
  if (!form.value.name.trim() || !form.value.spec.trim()) return null
  if (isEdit.value && punchUniqueKey(normalizedPunchForm.value) === originalUniqueKey.value) return null
  return findPunchDuplicate(
    normalizedPunchForm.value,
    punchList.value,
    isEdit.value ? form.value.id : '',
  )
})

const showLinkedScrewsDialog = ref(false)
const linkedScrews = ref<any[]>([])
const linkedPunchName = ref('')
const linkedLoading = ref(false)

async function showLinkedScrews(punch: any) {
  linkedPunchName.value = punch.name
  linkedScrews.value = []
  linkedLoading.value = true
  showLinkedScrewsDialog.value = true
  try {
    // 找到所有关联该冲头的记录
    const links = linkList.value.filter(l => l.punchId === punch.id)
    // 获取关联的螺丝规格
    const screwIds = links.map(l => l.screwSpecId)
    linkedScrews.value = screwSpecList.value.filter(s => screwIds.includes(s.id))
  } catch (error) {
    showDetailedError('加载冲头关联螺丝', error)
  } finally {
    linkedLoading.value = false
  }
}

const showOrderDialog = ref(false)
const orderForm = ref({ punchId: '', quantity: 1, orderDate: getCurrentDateTime(), remark: '' })

const showUseDialog = ref(false)
const useForm = ref({ punchId: '', user: '', quantity: 1, useDate: getCurrentDateTime(), remark: '' })

const showLinkDialog = ref(false)
const linkForm = ref({ punchId: '', screwSpecId: '', remark: '' })

// 表单引用
const formRef = ref<FormInstance>()
const orderFormRef = ref<FormInstance>()
const useFormRef = ref<FormInstance>()
const linkFormRef = ref<FormInstance>()

// 验证规则
const formRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  spec: [{ required: true, message: '请输入规格', trigger: 'blur' }],
}
const orderFormRules = {
  punchId: [{ required: true, message: '请选择冲头', trigger: 'change' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const useFormRules = {
  punchId: [{ required: true, message: '请选择冲头', trigger: 'change' }],
  user: [{ required: true, message: '请输入领用人', trigger: 'blur' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const linkFormRules = {
  punchId: [{ required: true, message: '请选择冲头', trigger: 'change' }],
  screwSpecId: [{ required: true, message: '请选择螺丝规格', trigger: 'change' }],
}

onMounted(() => {
  loadData()
  loadSpecOptions()
})

async function loadSpecOptions() {
  try {
    const options = await punchSpecApi.get()
    if (options.length > 0) specOptions.value = options
  } catch (error) {
    showDetailedError('加载冲头规格列表', error)
  }
}

async function loadData() {
  loading.value = true
  try {
    const { values, failures } = await settleNamedRequests([
      { label: '冲头信息', request: punchApi.getAll() },
      { label: '入库记录', request: punchOrderApi.getAll() },
      { label: '领用记录', request: punchUseApi.getAll() },
      { label: '螺丝规格关联', request: punchLinkApi.getAll() },
      { label: '螺丝规格信息', request: screwSpecApi.getAll() },
      { label: '库存计算', request: stockCalcApi.calculate('punch') },
    ])
    const [punches, orders, uses, links, screwSpecs, stockData] = values as Array<any[] | undefined>

    if (punches) {
      const stockMap: Record<string, any> = {}
      stockData?.forEach((item: any) => { stockMap[item.punchId] = item })
      punchList.value = punches.map((punch: any) => ({
        ...punch,
        currentStock: stockData ? (stockMap[punch.id]?.currentStock ?? '') : '',
        safetyStock: stockMap[punch.id]?.safetyStock ?? punch.safetyStock,
        status: stockData ? (stockMap[punch.id]?.status ?? '') : '',
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
    showBatchErrors('冲头数据加载', failures)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  originalUniqueKey.value = ''
  form.value = { id: '', name: '', spec: '', material: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  originalUniqueKey.value = punchUniqueKey(row)
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此冲头？', '提示', { type: 'warning' })
    await punchApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除冲头', error)
    }
  }
}

async function handleSubmit() {
  if (!formRef.value || submitting.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    if (duplicateMatch.value?.kind === 'exact') {
      ElMessage.error('已存在完全相同的冲头，请勿重复保存')
      return
    }

    submitting.value = true
    try {
      const payload = normalizedPunchForm.value
      if (isEdit.value) {
        await punchApi.update(form.value.id, payload)
        ElMessage.success('更新成功')
      } else {
        await punchApi.add(payload)
        ElMessage.success('添加成功')
      }
      dialogVisible.value = false
      await loadData()
    } catch (error) {
      showDetailedError(
        isEdit.value ? '更新冲头' : '添加冲头',
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
      await punchOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { punchId: '', quantity: 1, orderDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加冲头入库记录', error)
    }
  })
}

async function handleUseSubmit() {
  if (!useFormRef.value) return
  await useFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await punchUseApi.add(useForm.value)
      ElMessage.success('领用记录添加成功')
      showUseDialog.value = false
      useForm.value = { punchId: '', user: '', quantity: 1, useDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加冲头领用记录', error)
    }
  })
}

async function handleLinkSubmit() {
  if (!linkFormRef.value) return
  await linkFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await punchLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { punchId: '', screwSpecId: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加冲头螺丝规格关联', error)
    }
  })
}

async function handleDeleteLink(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此关联？', '提示', { type: 'warning' })
    await punchLinkApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除冲头螺丝规格关联', error)
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