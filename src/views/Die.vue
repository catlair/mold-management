<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Grid /></el-icon>
          <span>牙板管理</span>
          <el-button type="primary" @click="handleAdd">
            <el-icon><Plus /></el-icon>
            添加牙板
          </el-button>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="牙板信息" name="info">
          <el-table :data="dieList" border style="width: 100%" v-loading="loading">
            <el-table-column prop="name" label="名称" width="180" sortable />
            <el-table-column prop="machineType" label="机型" width="140" sortable :filters="machineTypeFilters" :filter-method="filterHandler" />
            <el-table-column prop="wireDiameter" label="线径" width="120" sortable />
            <el-table-column prop="safetyStock" label="安全库存" width="120" sortable />
            <el-table-column prop="remark" label="备注" min-width="150" />
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="handleEdit(row)">编辑</el-button>
                <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="订购记录" name="order">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showOrderDialog = true">新增订购</el-button>
          </div>
          <el-table :data="orderList" border style="width: 100%">
            <el-table-column label="牙板" width="120" sortable>
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="订购数量" width="100" sortable />
            <el-table-column prop="orderDate" label="订购时间" width="120" sortable />
            <el-table-column prop="status" label="到货状态" width="100" sortable :filters="statusFilters" :filter-method="filterHandler" />
            <el-table-column prop="remark" label="备注" />
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="领用记录" name="use">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showUseDialog = true">新增领用</el-button>
          </div>
          <el-table :data="useList" border style="width: 100%">
            <el-table-column label="牙板" width="120" sortable>
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
              </template>
            </el-table-column>
            <el-table-column prop="user" label="领用人" width="100" sortable />
            <el-table-column prop="quantity" label="领用数量" width="100" sortable />
            <el-table-column prop="useDate" label="领用时间" width="120" sortable />
            <el-table-column prop="remark" label="备注" />
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="库存汇总" name="stock">
          <el-table :data="stockList" border style="width: 100%">
            <el-table-column prop="name" label="牙板名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" sortable />
            <el-table-column prop="safetyStock" label="安全库存" width="100" sortable />
            <el-table-column prop="status" label="库存状态" width="100" :filters="stockStatusFilters" :filter-method="filterHandler">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="螺丝规格关联" name="link">
          <div class="tab-header">
            <el-button type="primary" size="small" @click="showLinkDialog = true">新增关联</el-button>
          </div>
          <el-table :data="linkList" border style="width: 100%">
            <el-table-column label="牙板" width="200">
              <template #default="{ row }">
                {{ getDieName(row.dieId) }}
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

    <!-- 订购对话框 -->
    <el-dialog v-model="showOrderDialog" title="新增订购" width="500px">
      <el-form ref="orderFormRef" :model="orderForm" :rules="orderFormRules" label-width="80px">
        <el-form-item label="牙板">
          <el-select v-model="orderForm.dieId" placeholder="请选择">
            <el-option v-for="item in dieList" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="数量">
          <el-input-number v-model="orderForm.quantity" :min="1" />
        </el-form-item>
        <el-form-item label="订购时间">
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

    <!-- 领用对话框 -->
    <el-dialog v-model="showUseDialog" title="新增领用" width="500px">
      <el-form ref="useFormRef" :model="useForm" :rules="useFormRules" label-width="80px">
        <el-form-item label="牙板">
          <el-select v-model="useForm.dieId" placeholder="请选择">
            <el-option v-for="item in dieList" :key="item.id" :label="item.name" :value="item.id" />
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { dieApi, dieOrderApi, dieUseApi, dieLinkApi, screwSpecApi, stockCalcApi } from '../api'

function getCurrentDateTime() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

const activeTab = ref('info')
const dieList = ref<any[]>([])
const orderList = ref<any[]>([])
const useList = ref<any[]>([])
const stockList = ref<any[]>([])
const linkList = ref<any[]>([])
const screwSpecList = ref<any[]>([])
const loading = ref(true)

// 筛选选项
const machineTypeFilters = computed(() => {
  const types = [...new Set(dieList.value.map(item => item.machineType).filter(Boolean))]
  return types.map(t => ({ text: t, value: t }))
})

const statusFilters = [
  { text: '未到货', value: '未到货' },
  { text: '已到货', value: '已到货' }
]

const stockStatusFilters = [
  { text: '需订购', value: '需订购' },
  { text: '安全', value: '安全' }
]

function filterHandler(value: string, row: any, column: any) {
  const property = column.property
  return row[property] === value
}

// 获取牙板名称
function getDieName(dieId: string) {
  const die = dieList.value.find(d => d.id === dieId)
  return die ? `${die.name} (${die.machineType})` : dieId
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
const orderForm = ref({ dieId: '', quantity: 1, orderDate: getCurrentDateTime(), status: '未到货', remark: '' })

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
})

async function loadData() {
  loading.value = true
  try {
    const [dies, orders, uses, links, screwSpecs] = await Promise.all([
      dieApi.getAll(),
      dieOrderApi.getAll(),
      dieUseApi.getAll(),
      dieLinkApi.getAll(),
      screwSpecApi.getAll()
    ])
    dieList.value = dies
    orderList.value = orders
    useList.value = uses
    linkList.value = links
    screwSpecList.value = screwSpecs
    stockList.value = await stockCalcApi.calculate('die')
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
      ElMessage.success('订购记录添加成功')
      showOrderDialog.value = false
      orderForm.value = { dieId: '', quantity: 1, orderDate: '', status: '未到货', remark: '' }
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
.page-container {
  height: 100%;
}
</style>
