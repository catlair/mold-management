<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Box /></el-icon>
          <span>主模具管理</span>
          <div class="header-right">
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加主模具
            </el-button>
            <FullscreenToggle />
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="主模具信息" name="info">
           <DataTable :data="mainMoldList" :loading="loading">
            <vxe-column field="name" title="名称" width="160" sortable />
            <vxe-column field="holeDiameter" title="孔径" width="100" sortable />
            <vxe-column field="wireMaterial" title="对应线材" width="120" sortable />
            <vxe-column field="safetyStock" title="安全库存" width="100" sortable />
            <vxe-column field="currentStock" title="当前库存" width="100" sortable />
            <vxe-column field="status" title="库存状态" width="100" sortable>
              <template #default="{ row }">
                <el-tag v-if="row.status" :type="row.status === '需入库' ? 'danger' : 'success'" effect="dark" round size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </vxe-column>
            <vxe-column field="remark" title="备注" min-width="120" />
            <vxe-column title="操作" width="150">
              <template #default="{ row }">
                <el-button size="small" @click="handleEdit(row)">编辑</el-button>
                <el-button size="small" type="danger" v-if="allowDelete" @click="handleDelete(row)">删除</el-button>
              </template>
            </vxe-column>
          </DataTable>
        </el-tab-pane>

        <el-tab-pane label="入库记录" name="order">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增入库</el-button>
          </div>
          <vxe-table :data="orderPaginated" border style="width: 100%">

            <vxe-column title="主模具" width="140" sortable>
              <template #default="{ row }">
                {{ getMainMoldName(row.mainMoldId) }}
              </template>
            </vxe-column>
            <vxe-column field="quantity" title="入库数量" width="120" sortable />
            <vxe-column field="orderDate" title="入库时间" width="180" sortable />
            <vxe-column field="status" title="到货状态" width="120" sortable :filters="statusFilters" :filter-method="filterHandler" />
            <vxe-column field="remark" title="备注" min-width="150" />

</vxe-table>
          <el-pagination
            v-model:current-page="orderCurrentPage"
            v-model:page-size="orderPageSize"
            :page-sizes="[10, 20, 50]"
            :total="orderList.length"
            layout="total, sizes, prev, pager, next"
            small
            style="margin-top: 12px; justify-content: flex-end;"
          />
        </el-tab-pane>

        <el-tab-pane label="使用记录" name="use">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showUseDialog = true">新增使用</el-button>
          </div>
          <vxe-table :data="usePaginated" border style="width: 100%">

            <vxe-column title="主模具" width="140" sortable>
              <template #default="{ row }">
                {{ getMainMoldName(row.mainMoldId) }}
              </template>
            </vxe-column>
            <vxe-column field="user" title="使用人" width="120" sortable />
            <vxe-column field="quantity" title="使用数量" width="120" sortable />
            <vxe-column field="useDate" title="使用时间" width="180" sortable />
            <vxe-column field="remark" title="备注" min-width="120" />

</vxe-table>
          <el-pagination
            v-model:current-page="useCurrentPage"
            v-model:page-size="usePageSize"
            :page-sizes="[10, 20, 50]"
            :total="useList.length"
            layout="total, sizes, prev, pager, next"
            small
            style="margin-top: 12px; justify-content: flex-end;"
          />
        </el-tab-pane>

        <el-tab-pane label="线材关联" name="link">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <vxe-table :data="linkList" border style="width: 100%">
            <vxe-column title="主模具" width="120" sortable>
              <template #default="{ row }">
                {{ getMainMoldName(row.mainMoldId) }}
              </template>
            </vxe-column>
            <vxe-column field="wireMaterial" title="线材规格" width="120" sortable />
            <vxe-column field="remark" title="备注" />
            <vxe-column title="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" type="danger" v-if="allowDelete" @click="handleDeleteLink(row)">删除</el-button>
              </template>
            </vxe-column>
          </vxe-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑主模具' : '添加主模具'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="孔径" prop="holeDiameter">
          <el-input v-model="form.holeDiameter" />
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
        <el-form-item label="主模具">
          <el-select v-model="orderForm.mainMoldId" placeholder="请选择">
            <el-option v-for="item in mainMoldList" :key="item.id" :label="item.name" :value="item.id" />
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
        <el-form-item label="主模具">
          <el-select v-model="useForm.mainMoldId" placeholder="请选择">
            <el-option v-for="item in mainMoldList" :key="item.id" :label="item.name" :value="item.id" />
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
        <el-form-item label="主模具">
          <el-select v-model="linkForm.mainMoldId" placeholder="请选择">
            <el-option v-for="item in mainMoldList" :key="item.id" :label="item.name" :value="item.id" />
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
import { mainMoldApi, mainMoldOrderApi, mainMoldUseApi, mainMoldLinkApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
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
const mainMoldList = ref<any[]>([])
useHighlight(mainMoldList)
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

// 筛选选项
const statusFilters = [
  { label: '未到货', value: '未到货' },
  { label: '已到货', value: '已到货' }
]

function filterHandler({ value, row, column }: any) {
  const property = column.property
  return row[property] === value
}

// 获取主模具名称
function getMainMoldName(mainMoldId: string) {
  const mold = mainMoldList.value.find(m => m.id === mainMoldId)
  return mold ? mold.name : mainMoldId
}

const dialogVisible = ref(false)
const isEdit = ref(false)
const form = ref({ id: '', name: '', holeDiameter: '', wireMaterial: '', safetyStock: 0, remark: '' })

const showOrderDialog = ref(false)
const orderForm = ref({ mainMoldId: '', quantity: 1, orderDate: getCurrentDateTime(), status: '未到货', remark: '' })

const showUseDialog = ref(false)
const useForm = ref({ mainMoldId: '', user: '', quantity: 1, useDate: getCurrentDateTime(), remark: '' })

const showLinkDialog = ref(false)
const linkForm = ref({ mainMoldId: '', wireMaterial: '', remark: '' })

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
  mainMoldId: [{ required: true, message: '请选择主模具', trigger: 'change' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const useFormRules = {
  mainMoldId: [{ required: true, message: '请选择主模具', trigger: 'change' }],
  user: [{ required: true, message: '请输入使用人', trigger: 'blur' }],
  quantity: [{ required: true, message: '请输入数量', trigger: 'blur' }],
}
const linkFormRules = {
  mainMoldId: [{ required: true, message: '请选择主模具', trigger: 'change' }],
  wireMaterial: [{ required: true, message: '请输入线材规格', trigger: 'blur' }],
}

onMounted(() => {
  loadData()
})

async function loadData() {
  loading.value = true
  try {
    const [molds, orders, uses, links, stockData] = await Promise.all([
      mainMoldApi.getAll(),
      mainMoldOrderApi.getAll(),
      mainMoldUseApi.getAll(),
      mainMoldLinkApi.getAll(),
      stockCalcApi.calculate('mainMold')
    ])
    const stockMap: Record<string, any> = {}
    stockData.forEach((s: any) => { stockMap[s.mainMoldId] = s })
    mainMoldList.value = molds.map((m: any) => ({
      ...m,
      currentStock: stockMap[m.id]?.currentStock ?? '',
      safetyStock: stockMap[m.id]?.safetyStock ?? m.safetyStock,
      status: stockMap[m.id]?.status ?? '',
    }))
    orderList.value = orders
    orderCurrentPage.value = 1
    useList.value = uses
    useCurrentPage.value = 1
    linkList.value = links
  } catch (error) {
    ElMessage.error('加载数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  form.value = { id: '', name: '', holeDiameter: '', wireMaterial: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此主模具？', '提示', { type: 'warning' })
    await mainMoldApi.remove(row.id)
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
        await mainMoldApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await mainMoldApi.add(form.value)
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
      await mainMoldOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { mainMoldId: '', quantity: 1, orderDate: '', status: '未到货', remark: '' }
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
      await mainMoldUseApi.add(useForm.value)
      ElMessage.success('使用记录添加成功')
      showUseDialog.value = false
      useForm.value = { mainMoldId: '', user: '', quantity: 1, useDate: '', remark: '' }
      loadData()
    } catch (error) {
      ElMessage.error('添加失败')
      console.error(error)
    }
  })
}

async function handleLinkSubmit() {
  if (!linkFormRef.value) return
  await linkFormRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      await mainMoldLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { mainMoldId: '', wireMaterial: '', remark: '' }
      loadData()
    } catch (error) {
      ElMessage.error('添加失败')
      console.error(error)
    }
  })
}

async function handleDeleteLink(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此关联？', '提示', { type: 'warning' })
    await mainMoldLinkApi.remove(row.id)
    ElMessage.success('删除成功')
    loadData()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
      console.error(error)
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