<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><SetUp /></el-icon>
          <span>冲头管理</span>
          <div class="header-right">
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加冲头
            </el-button>
            <el-button @click="toggleFullscreen">
              <el-icon><FullScreen v-if="!isFullscreen" /><Close v-else /></el-icon>
              {{ isFullscreen ? '退出全屏' : '全屏' }}
            </el-button>
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="冲头信息" name="info">
          <el-table :data="punchList" border style="width: 100%" v-loading="loading">
            <el-table-column prop="name" label="名称" width="160" sortable>
              <template #default="{ row }">
                <el-link type="primary" :underline="false" @click="showLinkedScrews(row)">{{ row.name }}</el-link>
              </template>
            </el-table-column>
            <el-table-column prop="spec" label="规格" width="80" sortable />
            <el-table-column prop="material" label="材质" width="120" sortable :filters="materialFilters" :filter-method="filterHandler" />
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
                <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="入库记录" name="order">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增入库</el-button>
          </div>
          <el-table :data="orderList" border style="width: 100%">
            <el-table-column label="冲头" width="140" sortable>
              <template #default="{ row }">
                {{ getPunchName(row.punchId) }}
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="入库数量" width="120" sortable />
            <el-table-column prop="orderDate" label="入库时间" width="180" sortable />
            <el-table-column prop="status" label="到货状态" width="120" sortable :filters="statusFilters" :filter-method="filterHandler" />
            <el-table-column prop="remark" label="备注" min-width="150" />
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="领用记录" name="use">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showUseDialog = true">新增领用</el-button>
          </div>
          <el-table :data="useList" border style="width: 100%">
            <el-table-column label="冲头" width="140" sortable>
              <template #default="{ row }">
                {{ getPunchName(row.punchId) }}
              </template>
            </el-table-column>
            <el-table-column prop="user" label="领用人" width="120" sortable />
            <el-table-column prop="quantity" label="领用数量" width="120" sortable />
            <el-table-column prop="useDate" label="领用时间" width="180" sortable />
            <el-table-column prop="remark" label="备注" min-width="150" />
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="螺丝规格关联" name="link">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <el-table :data="linkList" border style="width: 100%">
            <el-table-column label="冲头" width="200">
              <template #default="{ row }">
                {{ getPunchName(row.punchId) }}
              </template>
            </el-table-column>
            <el-table-column label="螺丝规格" width="200">
              <template #default="{ row }">
                {{ getScrewSpecName(row.screwSpecId) }}
              </template>
            </el-table-column>
            <el-table-column prop="remark" label="备注" />
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" type="danger" @click="handleDeleteLink(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 关联螺丝对话框 -->
    <el-dialog v-model="showLinkedScrewsDialog" :title="`关联螺丝 - ${linkedPunchName}`" width="700px">
      <el-table :data="linkedScrews" border style="width: 100%" v-loading="linkedLoading">
        <el-table-column prop="name" label="螺丝名称" width="150" sortable />
        <el-table-column prop="headType" label="头型" width="100" />
        <el-table-column prop="threadType" label="牙型" width="100" />
        <el-table-column prop="headSize" label="头/垫片大小" width="120" />
        <el-table-column prop="headHeight" label="头高" width="80" />
        <el-table-column prop="length" label="长度" width="80" />
        <el-table-column prop="threadDiameter" label="牙径" width="80" />
        <el-table-column prop="wireMaterial" label="线材" width="80" />
        <el-table-column prop="remark" label="备注" min-width="120" />
      </el-table>
      <div v-if="!linkedLoading && linkedScrews.length === 0" style="text-align: center; color: #909399; padding: 20px">
        该冲头暂无关联螺丝规格
      </div>
    </el-dialog>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑冲头' : '添加冲头'" width="500px">
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="80px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="规格" prop="spec">
          <el-input v-model="form.spec" />
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
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
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
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { punchApi, punchOrderApi, punchUseApi, punchLinkApi, screwSpecApi, stockCalcApi } from '../api'
import { toFullName } from '../utils/punchName'

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
const punchList = ref<any[]>([])
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const linkList = ref<any[]>([])
const screwSpecList = ref<any[]>([])
const loading = ref(true)

// 筛选选项
const materialFilters = computed(() => {
  const types = [...new Set(punchList.value.map(item => item.material).filter(Boolean))]
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
const form = ref({ id: '', name: '', spec: '', material: '', safetyStock: 0, remark: '' })

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
    ElMessage.error('加载关联数据失败')
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
})

async function loadData() {
  loading.value = true
  try {
    const [punches, orders, uses, links, screwSpecs, stockData] = await Promise.all([
      punchApi.getAll(),
      punchOrderApi.getAll(),
      punchUseApi.getAll(),
      punchLinkApi.getAll(),
      screwSpecApi.getAll(),
      stockCalcApi.calculate('punch')
    ])
    // 将库存数据合并到冲头列表
    const stockMap: Record<string, any> = {}
    stockData.forEach((s: any) => { stockMap[s.punchId] = s })
    punchList.value = punches.map((p: any) => ({
      ...p,
      currentStock: stockMap[p.id]?.currentStock ?? '',
      safetyStock: stockMap[p.id]?.safetyStock ?? p.safetyStock,
      status: stockMap[p.id]?.status ?? '',
    }))
    orderList.value = orders
    useList.value = uses
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
  form.value = { id: '', name: '', spec: '', material: '', safetyStock: 0, remark: '' }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
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
      // 简写自动转全写：30R → JMR M30
      const fullName = toFullName(form.value.name)
      const payload = { ...form.value, name: fullName || form.value.name }
      if (isEdit.value) {
        await punchApi.update(form.value.id, payload)
        ElMessage.success('更新成功')
      } else {
        await punchApi.add(payload)
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
      await punchOrderApi.add(orderForm.value)
      ElMessage.success('入库记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { punchId: '', quantity: 1, orderDate: '', remark: '' }
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
      await punchUseApi.add(useForm.value)
      ElMessage.success('领用记录添加成功')
      showUseDialog.value = false
      useForm.value = { punchId: '', user: '', quantity: 1, useDate: '', remark: '' }
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
      await punchLinkApi.add(linkForm.value)
      ElMessage.success('关联添加成功')
      showLinkDialog.value = false
      linkForm.value = { punchId: '', screwSpecId: '', remark: '' }
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
    await punchLinkApi.remove(row.id)
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
.page-container {
  height: 100%;
}
.stock-center {
  display: flex;
  justify-content: center;
}
.stock-center :deep(.el-table) {
  width: auto !important;
}
.page-container.is-fullscreen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 2000;
  background: #f0f2f5;
  padding: 20px;
  overflow: auto;
}
.page-container.is-fullscreen .el-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.page-container.is-fullscreen .el-card__body {
  flex: 1;
  overflow: auto;
}
.header-right {
  display: flex;
  gap: 8px;
  margin-left: auto;
}
</style>
