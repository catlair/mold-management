<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Scissor /></el-icon>
          <span>剪刀管理</span>
          <div class="header-right">
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加剪刀
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="剪刀信息" name="info">
           <DataTable table-id="scissor.info" :data="scissorList" :loading="loading">
            <ConfigurableTable field="name" title="名称" width="160" sortable />
            <ConfigurableTable field="diameter" title="口径" width="100" sortable />
            <ConfigurableTable field="wireMaterial" title="对应线材" width="120" sortable />
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
            <ConfigurableVxeTable table-id="scissor.order" class="record-table" :data="orderPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="剪刀" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getScissorName(row.scissorId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="quantity" title="入库数量" width="14%" min-width="120" sortable />
              <ConfigurableTable field="orderDate" title="入库时间" width="22%" min-width="180" sortable />
              <ConfigurableTable field="status" title="到货状态" width="16%" min-width="140" sortable :filters="statusFilters" :filter-method="filterHandler" />
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

        <el-tab-pane label="使用记录" name="use" class="record-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">使用记录</div>
              <div class="record-heading__count">共 {{ useList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showUseDialog = true">新增使用</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="scissor.use" class="record-table" :data="usePaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="剪刀" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getScissorName(row.scissorId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="user" title="使用人" width="16%" min-width="140" sortable />
              <ConfigurableTable field="quantity" title="使用数量" width="14%" min-width="120" sortable />
              <ConfigurableTable field="useDate" title="使用时间" width="22%" min-width="180" sortable />
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

        <el-tab-pane label="线材关联" name="link" class="association-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">线材关联</div>
              <div class="record-heading__count">共 {{ linkList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="scissor.link" class="record-table" :data="linkPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="剪刀" width="28%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getScissorName(row.scissorId) }}
                </template>
              </ConfigurableTable>
              <ConfigurableTable field="wireMaterial" title="线材规格" width="30%" min-width="220" sortable />
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
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑剪刀' : '添加剪刀'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="口径" prop="diameter">
          <el-input v-model="form.diameter" />
        </el-form-item>
        <el-form-item label="对应线材" prop="wireMaterial">
          <el-input v-model="form.wireMaterial" />
        </el-form-item>
        <el-form-item label="安全库存">
          <el-input-number v-model="form.safetyStock" :min="0" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 入库对话框 -->
    <el-dialog v-model="showOrderDialog" title="新增入库" width="500px">
      <el-form ref="orderFormRef" :model="orderForm" :rules="orderFormRules" label-width="80px">
        <el-form-item label="剪刀">
          <el-select v-model="orderForm.scissorId" placeholder="请选择">
            <el-option v-for="item in scissorList" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number v-model="orderForm.quantity" :min="1" />
        </el-form-item>
        <el-form-item label="入库时间">
          <el-date-picker v-model="orderForm.orderDate" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" />
        </el-form-item>
        <el-form-item label="到货状态">
          <el-select v-model="orderForm.status">
            <el-option label="未到货" value="未到货" />
            <el-option label="已到货" value="已到货" />
          </el-select>
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

    <!-- 使用对话框 -->
    <el-dialog v-model="showUseDialog" title="新增使用" width="500px">
      <el-form ref="useFormRef" :model="useForm" :rules="useFormRules" label-width="80px">
        <el-form-item label="剪刀">
          <el-select v-model="useForm.scissorId" placeholder="请选择">
            <el-option v-for="item in scissorList" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="使用人">
          <el-input v-model="useForm.user" />
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number v-model="useForm.quantity" :min="1" />
        </el-form-item>
        <el-form-item label="使用时间">
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
    <el-dialog v-model="showLinkDialog" title="新增线材关联" width="500px">
      <el-form ref="linkFormRef" :model="linkForm" :rules="linkFormRules" label-width="80px">
        <el-form-item label="剪刀">
          <el-select v-model="linkForm.scissorId" placeholder="请选择">
            <el-option v-for="item in scissorList" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="线材规格">
          <el-input v-model="linkForm.wireMaterial" />
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
import { scissorApi, scissorOrderApi, scissorUseApi, scissorLinkApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { settleNamedRequests, showBatchErrors, showDetailedError } from '../utils/errorFeedback'
import DataTable from '../components/DataTable.vue'
import FullscreenToggle from '../components/FullscreenToggle.vue'

const { allowDelete } = useAllowDelete()
// 全屏状态由 App 全局提供（isFullscreen 仅用于页面容器样式）
const { isFullscreen } = inject<any>('fullscreen')!

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const activeTab = ref('info')
const scissorList = ref<any[]>([])
useHighlight(scissorList)
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const linkList = ref<any[]>([])
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

// 筛选选项
const statusFilters = [
  { label: '未到货', value: '未到货' },
  { label: '已到货', value: '已到货' }
]

function filterHandler({ value, row, column }: any) {
  const property = column.property
  return row[property] === value
}

// 获取剪刀名称
function getScissorName(scissorId: string) {
  const scissor = scissorList.value.find(s => s.id === scissorId)
  return scissor ? scissor.name : scissorId
}

const dialogVisible = ref(false)
const isEdit = ref(false)
const form = ref({ id: '', name: '', diameter: '', wireMaterial: '', safetyStock: 0, remark: '' })

const showOrderDialog = ref(false)
const orderForm = ref({ scissorId: '', quantity: 1, orderDate: getCurrentDateTime(), status: '未到货', remark: '' })

const showUseDialog = ref(false)
const useForm = ref({ scissorId: '', user: '', quantity: 1, useDate: getCurrentDateTime(), remark: '' })

const showLinkDialog = ref(false)
const linkForm = ref({ scissorId: '', wireMaterial: '', remark: '' })

// 表单引用
const formRef = ref<FormInstance>()
const orderFormRef = ref<FormInstance>()
const useFormRef = ref<FormInstance>()
const linkFormRef = ref<FormInstance>()

// 验证规则
const formRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
}
const orderFormRules = {
  scissorId: [{ required: true, message: '请选择剪刀', trigger: 'change' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const useFormRules = {
  scissorId: [{ required: true, message: '请选择剪刀', trigger: 'change' }],
  user: [{ required: true, message: '请输入使用人', trigger: 'blur' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const linkFormRules = {
  scissorId: [{ required: true, message: '请选择剪刀', trigger: 'change' }],
  wireMaterial: [{ required: true, message: '请输入线材规格', trigger: 'blur' }],
}

onMounted(() => {
  loadData()
})

async function loadData() {
  loading.value = true
  try {
    const { values, failures } = await settleNamedRequests([
      { label: '剪刀信息', request: scissorApi.getAll() },
      { label: '入库记录', request: scissorOrderApi.getAll() },
      { label: '使用记录', request: scissorUseApi.getAll() },
      { label: '线材关联', request: scissorLinkApi.getAll() },
      { label: '库存计算', request: stockCalcApi.calculate('scissor') },
    ])
    const [scissors, orders, uses, links, stockData] = values as Array<any[] | undefined>

    if (scissors) {
      const stockMap: Record<string, any> = {}
      stockData?.forEach((item: any) => { stockMap[item.scissorId] = item })
      scissorList.value = scissors.map((scissor: any) => ({
        ...scissor,
        currentStock: stockData ? (stockMap[scissor.id]?.currentStock ?? '') : '',
        safetyStock: stockMap[scissor.id]?.safetyStock ?? scissor.safetyStock,
        status: stockData ? (stockMap[scissor.id]?.status ?? '') : '',
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
    showBatchErrors('剪刀数据加载', failures)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  form.value = { id: '', name: '', diameter: '', wireMaterial: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此剪刀？', '提示', { type: 'warning' })
    await scissorApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除剪刀', error)
    }
  }
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      if (isEdit.value) {
        await scissorApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await scissorApi.add(form.value)
        ElMessage.success('添加成功')
      }
      dialogVisible.value = false
      loadData()
    } catch (error) {
      showDetailedError(isEdit.value ? '更新剪刀' : '添加剪刀', error)
    }
  })
}

async function handleOrderSubmit() {
  if (!orderFormRef.value) return
  await orderFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await scissorOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { scissorId: '', quantity: 1, orderDate: '', status: '未到货', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加剪刀入库记录', error)
    }
  })
}

async function handleUseSubmit() {
  if (!useFormRef.value) return
  await useFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await scissorUseApi.add(useForm.value)
      ElMessage.success('使用记录添加成功')
      showUseDialog.value = false
      useForm.value = { scissorId: '', user: '', quantity: 1, useDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加剪刀使用记录', error)
    }
  })
}

async function handleLinkSubmit() {
  if (!linkFormRef.value) return
  await linkFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await scissorLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { scissorId: '', wireMaterial: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加剪刀线材关联', error)
    }
  })
}

async function handleDeleteLink(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此关联？', '提示', { type: 'warning' })
    await scissorLinkApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除剪刀线材关联', error)
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
</style>