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
            <el-button @click="toggleFullscreen">
              <el-icon><FullScreen v-if="!isFullscreen" /><Close v-else /></el-icon>
              {{ isFullscreen ? '退出全屏' : '全屏' }}
            </el-button>
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="皮带信息" name="info">
           <el-table :data="beltList" border style="width: 100%" max-height="calc(100vh - 200px)" v-loading="loading">
            <el-table-column prop="name" label="名称" width="160" sortable />
            <el-table-column prop="machine" label="适用机器" width="120" sortable :filters="machineFilters" :filter-method="filterHandler" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" sortable />
            <el-table-column prop="currentStock" label="当前库存" width="100" sortable />
            <el-table-column prop="status" label="库存状态" width="100" sortable>
              <template #default="{ row }">
                <el-tag v-if="row.status" :type="row.status === '需入库' ? 'danger' : 'success'" effect="dark" round size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="remark" label="备注" min-width="120" />
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="handleEdit(row)">编辑</el-button>
                <el-button size="small" type="danger" v-if="allowDelete" @click="handleDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="入库记录" name="order">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增入库</el-button>
          </div>
          <el-table :data="orderList" border style="width: 100%">
            <el-table-column label="皮带" width="140" sortable>
              <template #default="{ row }">
                {{ getBeltName(row.beltId) }}
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="入库数量" width="120" sortable />
            <el-table-column prop="orderDate" label="入库时间" width="180" sortable />
            <el-table-column prop="status" label="到货状态" width="120" sortable :filters="statusFilters" :filter-method="filterHandler" />
            <el-table-column prop="remark" label="备注" min-width="150" />
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="使用记录" name="use">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showUseDialog = true">新增使用</el-button>
          </div>
          <el-table :data="useList" border style="width: 100%">
            <el-table-column label="皮带" width="140" sortable>
              <template #default="{ row }">
                {{ getBeltName(row.beltId) }}
              </template>
            </el-table-column>
            <el-table-column prop="user" label="使用人" width="120" sortable />
            <el-table-column prop="quantity" label="使用数量" width="120" sortable />
            <el-table-column prop="useDate" label="使用时间" width="180" sortable />
            <el-table-column prop="remark" label="备注" min-width="150" />
          </el-table>
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
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { beltApi, beltOrderApi, beltUseApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'

const { allowDelete } = useAllowDelete()

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const isFullscreen = ref(false)

async function toggleFullscreen() {
  const next = !isFullscreen.value
  isFullscreen.value = next
  try {
    const win = getCurrentWindow()
    await win.setFullscreen(next)
  } catch {}
}

onMounted(async () => {
  try {
    const win = getCurrentWindow()
    isFullscreen.value = await win.isFullscreen()
  } catch {}
})

const activeTab = ref('info')
const beltList = ref<any[]>([])
useHighlight(beltList)
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const loading = ref(true)

// 筛选选项
const machineFilters = computed(() => {
  const types = [...new Set(beltList.value.map(item => item.machine).filter(Boolean))]
  return types.map(t => ({ text: t, value: t }))
})

const statusFilters = [
  { text: '未到货', value: '未到货' },
  { text: '已到货', value: '已到货' }
]

function filterHandler(value: string, row: any, column: any) {
  const property = column.property
  return row[property] === value
}

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
    const [belts, orders, uses, stockData] = await Promise.all([
      beltApi.getAll(),
      beltOrderApi.getAll(),
      beltUseApi.getAll(),
      stockCalcApi.calculate('belt')
    ])
    const stockMap: Record<string, any> = {}
    stockData.forEach((s: any) => { stockMap[s.beltId] = s })
    beltList.value = belts.map((b: any) => ({
      ...b,
      currentStock: stockMap[b.id]?.currentStock ?? '',
      safetyStock: stockMap[b.id]?.safetyStock ?? b.safetyStock,
      status: stockMap[b.id]?.status ?? '',
    }))
    orderList.value = orders
    useList.value = uses
  } catch (error) {
    ElMessage.error('加载数据失败')
    console.error(error)
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
      ElMessage.error('删除失败')
      console.error(error)
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
      ElMessage.error(isEdit.value ? '更新失败' : '添加失败')
      console.error(error)
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
      ElMessage.error('添加失败')
      console.error(error)
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
      ElMessage.error('添加失败')
      console.error(error)
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
.page-container.is-fullscreen .el-card__body { flex: 1; overflow: auto; padding: 12px; }
.header-right {
  display: flex;
  gap: 8px;
  margin-left: auto;
}
</style>
