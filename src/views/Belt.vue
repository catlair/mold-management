<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Connection /></el-icon>
          <span>皮带管理</span>
          <div class="header-right">
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加皮带
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="皮带信息" name="info">
           <DataTable table-id="belt.info" :data="beltList" :loading="loading">
            <ConfigurableTable field="name" title="名称" width="160" sortable />
            <ConfigurableTable field="machine" title="适用机器" width="120" sortable :filters="machineFilters" :filter-method="exactFilter" />
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
            <ConfigurableVxeTable table-id="belt.order" class="record-table" :data="orderPaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="皮带" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getBeltName(row.beltId) }}
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

        <el-tab-pane label="使用记录" name="use" class="record-pane">
          <div class="tab-header">
            <div class="record-heading">
              <div class="record-heading__title">使用记录</div>
              <div class="record-heading__count">共 {{ useList.length }} 条</div>
            </div>
            <el-button type="primary" size="small" @click="showUseDialog = true">新增使用</el-button>
          </div>
          <div class="record-table-scroll">
            <ConfigurableVxeTable table-id="belt.use" class="record-table" :data="usePaginated" border round stripe show-header-overflow="tooltip" style="width: 100%">
              <ConfigurableTable title="皮带" width="26%" min-width="220" sortable>
                <template #default="{ row }">
                  {{ getBeltName(row.beltId) }}
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
      </el-tabs>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑皮带' : '添加皮带'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="适用机器" prop="machine">
          <el-input v-model="form.machine" />
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
        <el-form-item label="皮带">
          <el-select v-model="orderForm.beltId" placeholder="请选择">
            <el-option v-for="item in beltList" :key="item.id" :label="item.name" :value="item.id" />
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
        <el-form-item label="皮带">
          <el-select v-model="useForm.beltId" placeholder="请选择">
            <el-option v-for="item in beltList" :key="item.id" :label="item.name" :value="item.id" />
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { beltApi, beltOrderApi, beltUseApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { showBatchErrors, showDetailedError, type NamedError } from '../utils/errorFeedback'
import DataTable from '../components/DataTable.vue'
import FullscreenToggle from '../components/FullscreenToggle.vue'
import { exactFilter, STATUS_FILTERS, buildFilters } from '../utils/tableFilters'

const { allowDelete } = useAllowDelete()
// 全屏状态由 App 全局提供（isFullscreen 仅用于页面容器样式）
const { isFullscreen } = inject<any>('fullscreen')!

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const activeTab = ref('info')
const beltList = ref<any[]>([])
useHighlight(beltList)
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
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

// 筛选选项
const machineFilters = computed(() => buildFilters(beltList.value, 'machine'))

// 获取皮带名称
function getBeltName(beltId: string) {
  const belt = beltList.value.find(b => b.id === beltId)
  return belt ? belt.name : beltId
}

const dialogVisible = ref(false)
const isEdit = ref(false)
const form = ref({ id: '', name: '', machine: '', safetyStock: 0, remark: '' })

const showOrderDialog = ref(false)
const orderForm = ref({ beltId: '', quantity: 1, orderDate: getCurrentDateTime(), status: '未到货', remark: '' })

const showUseDialog = ref(false)
const useForm = ref({ beltId: '', user: '', quantity: 1, useDate: getCurrentDateTime(), remark: '' })

// 表单引用
const formRef = ref<FormInstance>()
const orderFormRef = ref<FormInstance>()
const useFormRef = ref<FormInstance>()

// 验证规则
const formRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  machine: [{ required: true, message: '请输入适用机器', trigger: 'blur' }],
}
const orderFormRules = {
  beltId: [{ required: true, message: '请选择皮带', trigger: 'change' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const useFormRules = {
  beltId: [{ required: true, message: '请选择皮带', trigger: 'change' }],
  user: [{ required: true, message: '请输入使用人', trigger: 'blur' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}

onMounted(() => {
  loadData()
})

async function loadData() {
  loading.value = true
  try {
    const requests = [
      { label: '皮带信息', request: beltApi.getAll() },
      { label: '入库记录', request: beltOrderApi.getAll() },
      { label: '使用记录', request: beltUseApi.getAll() },
      { label: '库存计算', request: stockCalcApi.calculate('belt') },
    ]
    const results = await Promise.allSettled(requests.map(item => item.request))
    const failures: NamedError[] = []
    results.forEach((result, index) => {
      if (result.status === 'rejected') failures.push({ label: requests[index].label, error: result.reason })
    })

    const belts = results[0].status === 'fulfilled' ? results[0].value : null
    const orders = results[1].status === 'fulfilled' ? results[1].value : null
    const uses = results[2].status === 'fulfilled' ? results[2].value : null
    const stockData = results[3].status === 'fulfilled' ? results[3].value : null

    if (belts) {
      const stockMap: Record<string, any> = {}
      stockData?.forEach((item: any) => { stockMap[item.beltId] = item })
      beltList.value = belts.map((belt: any) => ({
        ...belt,
        currentStock: stockData ? (stockMap[belt.id]?.currentStock ?? '') : '',
        safetyStock: stockMap[belt.id]?.safetyStock ?? belt.safetyStock,
        status: stockData ? (stockMap[belt.id]?.status ?? '') : '',
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
    showBatchErrors('皮带数据加载', failures)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  form.value = { id: '', name: '', machine: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此皮带？', '提示', { type: 'warning' })
    await beltApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      showDetailedError('删除皮带', error)
    }
  }
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      if (isEdit.value) {
        await beltApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await beltApi.add(form.value)
        ElMessage.success('添加成功')
      }
      dialogVisible.value = false
      loadData()
    } catch (error) {
      showDetailedError(isEdit.value ? '更新皮带' : '添加皮带', error)
    }
  })
}

async function handleOrderSubmit() {
  if (!orderFormRef.value) return
  await orderFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await beltOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { beltId: '', quantity: 1, orderDate: '', status: '未到货', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加皮带入库记录', error)
    }
  })
}

async function handleUseSubmit() {
  if (!useFormRef.value) return
  await useFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await beltUseApi.add(useForm.value)
      ElMessage.success('使用记录添加成功')
      showUseDialog.value = false
      useForm.value = { beltId: '', user: '', quantity: 1, useDate: '', remark: '' }
      loadData()
    } catch (error) {
      showDetailedError('添加皮带使用记录', error)
    }
  })
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