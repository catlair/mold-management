<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>螺丝规格管理</span>
          <el-button type="primary" @click="handleAdd">
            <el-icon><Plus /></el-icon>
            添加规格
          </el-button>
        </div>
      </template>

      <el-table :data="tableData" border style="width: 100%" max-height="600" v-loading="loading">
        <el-table-column prop="name" label="螺丝名称" width="150" sortable />
        <el-table-column prop="headType" label="头型" width="100" sortable :filters="headTypeFilters" :filter-method="filterHandler" />
        <el-table-column prop="punch" label="冲头" width="100" sortable />
        <el-table-column prop="threadType" label="牙型" width="100" sortable :filters="threadTypeFilters" :filter-method="filterHandler" />
        <el-table-column prop="die" label="牙板" width="100" sortable />
        <el-table-column prop="headSize" label="头/垫片大小" width="110" sortable />
        <el-table-column prop="headHeight" label="头高" width="80" sortable />
        <el-table-column prop="length" label="长度" width="80" sortable />
        <el-table-column prop="threadDiameter" label="牙径" width="80" sortable />
        <el-table-column prop="shankLength" label="光钉长度" width="100" sortable />
        <el-table-column prop="wireMaterial" label="线材" width="80" sortable />
        <el-table-column prop="plating" label="电镀" width="80" sortable :filters="platingFilters" :filter-method="filterHandler" />
        <el-table-column prop="customer" label="客户名" width="100" sortable />
        <el-table-column prop="externalId" label="外部ID" width="100" sortable />
        <el-table-column prop="remark" label="备注" min-width="120" />
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="handleEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div v-if="!loading && tableData.length === 0" class="empty-state">
        <el-empty description="暂无数据" />
      </div>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑螺丝规格' : '添加螺丝规格'"
      width="800px"
    >
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="100px">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="螺丝名称" prop="name">
              <el-input v-model="form.name" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="螺丝头型">
              <el-input v-model="form.headType" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="冲头">
              <el-select v-model="form.punch" placeholder="请选择冲头" filterable allow-create>
                <el-option
                  v-for="item in punchOptions"
                  :key="item.name"
                  :label="item.name"
                  :value="item.name"
                >
                  <span>{{ item.name }}</span>
                  <span style="float: right; color: #8492a6; font-size: 12px">{{ item.specs }}</span>
                </el-option>
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙型">
              <el-input v-model="form.threadType" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙板">
              <el-select v-model="form.die" placeholder="请选择牙板" filterable allow-create>
                <el-option
                  v-for="item in dieOptions"
                  :key="item.name"
                  :label="item.name"
                  :value="item.name"
                >
                  <span>{{ item.name }}</span>
                  <span style="float: right; color: #8492a6; font-size: 12px">{{ item.specs }}</span>
                </el-option>
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="头/垫片大小">
              <el-input v-model="form.headSize" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="头高">
              <el-input v-model="form.headHeight" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="长度">
              <el-input v-model="form.length" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="牙径">
              <el-input v-model="form.threadDiameter" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="光钉长度">
              <el-input v-model="form.shankLength" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="线材">
              <el-input v-model="form.wireMaterial" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="电镀">
              <el-input v-model="form.plating" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="客户名">
              <el-input v-model="form.customer" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="外部ID">
              <el-input v-model="form.externalId" />
            </el-form-item>
          </el-col>
          <el-col :span="24">
            <el-form-item label="备注">
              <el-input v-model="form.remark" type="textarea" />
            </el-form-item>
          </el-col>
        </el-row>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSubmit">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { screwSpecApi, punchApi, dieApi } from '../api'

const tableData = ref<any[]>([])
const punchList = ref<any[]>([])
const dieList = ref<any[]>([])
const dialogVisible = ref(false)
const isEdit = ref(false)
const loading = ref(true)

// 表单引用
const formRef = ref<FormInstance>()

// 验证规则
const formRules = {
  name: [{ required: true, message: '请输入螺丝名称', trigger: 'blur' }],
}

// 冲头选项（按名称去重）
const punchOptions = computed(() => {
  const names = [...new Set(punchList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => {
    const items = punchList.value.filter(p => p.name === name)
    return {
      name,
      specs: items.map(p => `${p.spec}${p.material ? '(' + p.material + ')' : ''}`).join('、')
    }
  })
})

// 牙板选项（按名称去重）
const dieOptions = computed(() => {
  const names = [...new Set(dieList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => {
    const items = dieList.value.filter(d => d.name === name)
    return {
      name,
      specs: items.map(d => `${d.machineType}${d.wireDiameter ? '(' + d.wireDiameter + ')' : ''}`).join('、')
    }
  })
})

// 筛选选项
const headTypeFilters = computed(() => {
  const types = [...new Set(tableData.value.map(item => item.headType).filter(Boolean))]
  return types.map(t => ({ text: t, value: t }))
})

const threadTypeFilters = computed(() => {
  const types = [...new Set(tableData.value.map(item => item.threadType).filter(Boolean))]
  return types.map(t => ({ text: t, value: t }))
})

const platingFilters = computed(() => {
  const types = [...new Set(tableData.value.map(item => item.plating).filter(Boolean))]
  return types.map(t => ({ text: t, value: t }))
})

function filterHandler(value: string, row: any, column: any) {
  const property = column.property
  return row[property] === value
}

const form = ref({
  id: '',
  customer: '',
  externalId: '',
  name: '',
  headType: '',
  punch: '',
  threadType: '',
  die: '',
  headSize: '',
  headHeight: '',
  length: '',
  threadDiameter: '',
  shankLength: '',
  wireMaterial: '',
  plating: '',
  remark: ''
})

onMounted(() => {
  loadData()
})

async function loadData() {
  loading.value = true
  try {
    const [screws, punches, dies] = await Promise.all([
      screwSpecApi.getAll(),
      punchApi.getAll(),
      dieApi.getAll()
    ])
    tableData.value = screws
    punchList.value = punches
    dieList.value = dies
  } catch (error) {
    ElMessage.error('加载数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

function handleAdd() {
  isEdit.value = false
  form.value = {
    id: '',
    customer: '',
    externalId: '',
    name: '',
    headType: '',
    punch: '',
    threadType: '',
    die: '',
    headSize: '',
    headHeight: '',
    length: '',
    threadDiameter: '',
    shankLength: '',
    wireMaterial: '',
    plating: '',
    remark: ''
  }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = { ...row }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此规格？', '提示', {
      type: 'warning'
    })
    await screwSpecApi.remove(row.id)
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
        await screwSpecApi.update(form.value.id, form.value)
        ElMessage.success('更新成功')
      } else {
        await screwSpecApi.add(form.value)
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
</script>

<style scoped>
.page-container {
  height: 100%;
}
</style>
