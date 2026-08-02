<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Grid /></el-icon>
          <span>牙板管理</span>
          <div class="header-right">
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
           <DataTable :data="dieList" :loading="loading">
            <vxe-column field="name" title="名称" width="160" sortable>
              <template #default="{ row }">
                <el-link type="primary" :underline="false" @click="showLinkedScrews(row)">{{ row.name }}</el-link>
              </template>
            </vxe-column>
            <vxe-column field="machineType" title="机型" width="120" sortable :filters="machineTypeFilters" :filter-method="filterHandler" />
            <vxe-column field="wireDiameter" title="线径" width="100" sortable />
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

        <el-tab-pane label="入库记录" name="order" class="record-pane">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增入库</el-button>
          </div>
          <div class="record-table-scroll">
            <vxe-table :data="orderPaginated" border style="width: 100%">

            <vxe-column title="牙板名称" width="140" sortable>
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
              </template>
            </vxe-column>
            <vxe-column field="quantity" title="入库数量" width="100" sortable />
            <vxe-column field="orderDate" title="入库时间" width="180" sortable />
            <vxe-column field="remark" title="备注" min-width="120" />

            </vxe-table>
          </div>
          <el-pagination
            v-model:current-page="orderCurrentPage"
            v-model:page-size="orderPageSize"
            :page-sizes="[10, 20, 50]"
            :total="orderList.length"
            layout="total, sizes, prev, pager, next"
            small
            class="record-pagination"
          />
        </el-tab-pane>

        <el-tab-pane label="领用记录" name="use" class="record-pane">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showUseDialog = true">新增领用</el-button>
          </div>
          <div class="record-table-scroll">
            <vxe-table :data="usePaginated" border style="width: 100%">

            <vxe-column title="牙板名称" width="140" sortable>
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
              </template>
            </vxe-column>
            <vxe-column field="user" title="领用人" width="120" sortable />
            <vxe-column field="quantity" title="领用数量" width="120" sortable />
            <vxe-column field="useDate" title="领用时间" width="180" sortable />
            <vxe-column field="remark" title="备注" min-width="150" />

            </vxe-table>
          </div>
          <el-pagination
            v-model:current-page="useCurrentPage"
            v-model:page-size="usePageSize"
            :page-sizes="[10, 20, 50]"
            :total="useList.length"
            layout="total, sizes, prev, pager, next"
            small
            class="record-pagination"
          />
        </el-tab-pane>

        <el-tab-pane label="螺丝规格关联" name="link" class="association-pane">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <div class="record-table-scroll">
            <vxe-table :data="linkList" border style="width: 100%">
            <vxe-column title="牙板" width="200">
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
              </template>
            </vxe-column>
            <vxe-column title="螺丝规格" width="200">
              <template #default="{ row }">
                {{ getScrewSpecName(row.screwSpecId) }}
              </template>
            </vxe-column>
            <vxe-column field="remark" title="备注" />
            <vxe-column title="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" type="danger" v-if="allowDelete" @click="handleDeleteLink(row)">删除</el-button>
              </template>
            </vxe-column>
            </vxe-table>
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
          <el-input v-model="form.machineType" />
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
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
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
      <vxe-table :data="linkedScrews" border round stripe style="width: 100%" :loading="linkedLoading">
        <vxe-column field="name" title="螺丝名称" width="150" sortable />
        <vxe-column field="headType" title="头型" width="100" />
        <vxe-column field="threadType" title="牙型" width="100" />
        <vxe-column field="headSize" title="头/垫片大小" width="120" />
        <vxe-column field="headHeight" title="头高" width="80" />
        <vxe-column field="length" title="长度" width="80" />
        <vxe-column field="threadDiameter" title="牙径" width="80" />
        <vxe-column field="wireMaterial" title="线材" width="80" />
        <vxe-column field="remark" title="备注" min-width="120" />
      </vxe-table>
      <div v-if="!linkedLoading && linkedScrews.length === 0" class="related-dialog-empty">
        该牙板暂无关联螺丝规格
      </div>
    </RelatedDataDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { dieApi, dieOrderApi, dieUseApi, dieLinkApi, screwSpecApi, stockCalcApi } from '../api'
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import DataTable from '../components/DataTable.vue'
import RelatedDataDialog from '../components/RelatedDataDialog.vue'
import FullscreenToggle from '../components/FullscreenToggle.vue'

const { allowDelete } = useAllowDelete()
// 全屏状态由 App 全局提供（isFullscreen 仅用于页面容器样式）
const { isFullscreen } = inject<any>('fullscreen')!

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
  } catch {
    ElMessage.error('加载关联数据失败')
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
const form = ref({ id: '', name: '', machineType: '', wireDiameter: '', safetyStock: 0, remark: '' })

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
})

async function loadData() {
  loading.value = true
  try {
    const [dies, orders, uses, links, screwSpecs, stockData] = await Promise.all([
      dieApi.getAll(),
      dieOrderApi.getAll(),
      dieUseApi.getAll(),
      dieLinkApi.getAll(),
      screwSpecApi.getAll(),
      stockCalcApi.calculate('die')
    ])
    const stockMap: Record<string, any> = {}
    stockData.forEach((s: any) => { stockMap[s.dieId] = s })
    dieList.value = dies.map((d: any) => ({
      ...d,
      currentStock: stockMap[d.id]?.currentStock ?? '',
      safetyStock: stockMap[d.id]?.safetyStock ?? d.safetyStock,
      status: stockMap[d.id]?.status ?? '',
    }))
    orderList.value = orders
    orderCurrentPage.value = 1
    useList.value = uses
    useCurrentPage.value = 1
    linkList.value = links
    screwSpecList.value = screwSpecs
  } catch (error) {
    ElMessage.error('加载数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  form.value = { id: '', name: '', machineType: '', wireDiameter: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
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
        await dieApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await dieApi.add(form.value)
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
      await dieOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { dieId: '', quantity: 1, orderDate: '', remark: '' }
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
      await dieUseApi.add(useForm.value)
      ElMessage.success('领用记录添加成功')
      showUseDialog.value = false
      useForm.value = { dieId: '', user: '', quantity: 1, useDate: '', remark: '' }
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
      await dieLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { dieId: '', screwSpecId: '', remark: '' }
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
    await dieLinkApi.remove(row.id)
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