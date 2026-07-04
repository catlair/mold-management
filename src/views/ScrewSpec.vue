<template>
  <div class="page-container" :class="{ 'is-fullscreen': isFullscreen }">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>螺丝规格管理</span>
          <div class="header-right">
            <el-button type="primary" @click="handleAdd">
              <el-icon><Plus /></el-icon>
              添加规格
            </el-button>
            <el-button @click="toggleFullscreen">
              <el-icon><FullScreen v-if="!isFullscreen" /><Close v-else /></el-icon>
              {{ isFullscreen ? '退出全屏' : '全屏' }}
            </el-button>
          </div>
        </div>
      </template>

      <el-table :data="tableData" border style="width: 100%" v-loading="loading">
        <el-table-column prop="name" label="螺丝名称" width="160" sortable />
        <el-table-column prop="headType" label="头型" width="120" sortable :filters="headTypeFilters" :filter-method="filterHandler" />
        <el-table-column prop="punch" label="冲头" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.punch" type="primary" :underline="false" @click="showPunchDialog(row)">{{ row.punch }}</el-link>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column prop="threadType" label="牙型" width="120" sortable :filters="threadTypeFilters" :filter-method="filterHandler" />
        <el-table-column prop="die" label="牙板" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.die" type="success" :underline="false" @click="showDieDialog(row)">{{ row.die }}</el-link>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column prop="headSize" label="头/垫片大小" width="140" sortable />
        <el-table-column prop="headHeight" label="头高" width="100" sortable />
        <el-table-column prop="length" label="长度" width="100" sortable />
        <el-table-column prop="threadDiameter" label="牙径" width="100" sortable />
        <el-table-column prop="shankLength" label="光钉长度" width="120" sortable />
        <el-table-column prop="wireMaterial" label="线材" width="100" sortable />
        <el-table-column prop="plating" label="电镀" width="120" sortable :filters="platingFilters" :filter-method="filterHandler" />
        <el-table-column prop="customer" label="客户名" width="120" sortable />
        <el-table-column prop="externalId" label="外部ID" width="120" sortable />
        <el-table-column prop="remark" label="备注" min-width="140" />
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
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑螺丝规格' : '添加螺丝规格'" width="800px">
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
            <el-form-item label="冲头" prop="punch">
              <el-select v-model="form.punch" placeholder="请选择冲头" filterable allow-create multiple collapse-tags>
                <el-option v-for="item in punchOptions" :key="item.name" :label="item.name" :value="item.name">
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
            <el-form-item label="牙板" prop="die">
              <el-select v-model="form.die" placeholder="请选择牙板" filterable allow-create multiple collapse-tags>
                <el-option v-for="item in dieOptions" :key="item.name" :label="item.name" :value="item.name">
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
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 冲头关联弹窗 -->
    <el-dialog v-model="punchDialogVisible" title="冲头关联" width="700px">
      <el-table :data="punchDialogItems" border size="small">
        <el-table-column prop="name" label="冲头名称" width="100" />
        <el-table-column prop="spec" label="规格" width="100" />
        <el-table-column prop="material" label="材质" width="80" />
        <el-table-column label="当前库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.currentStock ?? '-' }}</template>
        </el-table-column>
        <el-table-column label="安全库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.safetyStock ?? '-' }}</template>
        </el-table-column>
        <el-table-column label="状态" width="90" align="center">
          <template #default="{ row: r }">
            <el-tag v-if="r.status" :type="r.status === '需入库' ? 'danger' : 'success'" size="small" round>{{ r.status }}</el-tag>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="外显" width="60" align="center">
          <template #default="{ row: item }">
            <el-link :type="item.name === punchDialogPrimary ? 'info' : 'warning'" :underline="false" @click="setPunchPrimary(item)">
              <el-icon><View /></el-icon>
            </el-link>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 牙板关联弹窗 -->
    <el-dialog v-model="dieDialogVisible" title="牙板关联" width="700px">
      <el-table :data="dieDialogItems" border size="small">
        <el-table-column prop="name" label="牙板名称" width="100" />
        <el-table-column prop="machineType" label="机型" width="100" />
        <el-table-column prop="wireDiameter" label="线径" width="80" />
        <el-table-column label="当前库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.currentStock ?? '-' }}</template>
        </el-table-column>
        <el-table-column label="安全库存" width="90" align="center">
          <template #default="{ row: r }">{{ r.safetyStock ?? '-' }}</template>
        </el-table-column>
        <el-table-column label="状态" width="90" align="center">
          <template #default="{ row: r }">
            <el-tag v-if="r.status" :type="r.status === '需入库' ? 'danger' : 'success'" size="small" round>{{ r.status }}</el-tag>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="外显" width="60" align="center">
          <template #default="{ row: item }">
            <el-link :type="item.name === dieDialogPrimary ? 'info' : 'warning'" :underline="false" @click="setDiePrimary(item)">
              <el-icon><View /></el-icon>
            </el-link>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { View } from '@element-plus/icons-vue'
import type { FormInstance } from 'element-plus'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { screwSpecApi, punchApi, dieApi, punchLinkApi, dieLinkApi, stockCalcApi } from '../api'

const tableData = ref<any[]>([])
const punchList = ref<any[]>([])
const dieList = ref<any[]>([])
const dialogVisible = ref(false)
const isEdit = ref(false)
const isFullscreen = ref(false)
const loading = ref(true)

// 关联弹窗状态
const punchDialogVisible = ref(false)
const punchDialogItems = ref<any[]>([])
const punchDialogPrimary = ref('')
const punchDialogRow = ref<any>({})
const dieDialogVisible = ref(false)
const dieDialogItems = ref<any[]>([])
const dieDialogPrimary = ref('')
const dieDialogRow = ref<any>({})

async function toggleFullscreen() {
  const next = !isFullscreen.value
  isFullscreen.value = next
  try { await getCurrentWindow().setFullscreen(next) } catch {}
}

onMounted(async () => {
  try { isFullscreen.value = await getCurrentWindow().isFullscreen() } catch {}
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && isFullscreen.value) toggleFullscreen() })
  loadData()
})

const formRef = ref<FormInstance>()
const formRules = { name: [{ required: true, message: '请输入螺丝名称', trigger: 'blur' }] }

// 解析逗号分隔字符串为数组
function parseNames(val: any): string[] {
  if (Array.isArray(val)) return val
  if (!val || typeof val !== 'string') return []
  const t = val.trim()
  if (t.startsWith('[')) { try { const a = JSON.parse(t); if (Array.isArray(a)) return a.map(String) } catch {} }
  return t.split(',').map(s => s.trim()).filter(Boolean)
}

// 冲头选项（按名称去重）
const punchOptions = computed(() => {
  const names = [...new Set(punchList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => ({
    name,
    specs: punchList.value.filter(p => p.name === name).map(p => `${p.spec}${p.material ? '(' + p.material + ')' : ''}`).join('、')
  }))
})

// 牙板选项（按名称去重）
const dieOptions = computed(() => {
  const names = [...new Set(dieList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => ({
    name,
    specs: dieList.value.filter(d => d.name === name).map(d => `${d.machineType}${d.wireDiameter ? '(' + d.wireDiameter + ')' : ''}`).join('、')
  }))
})

const headTypeFilters = computed(() => [...new Set(tableData.value.map(i => i.headType).filter(Boolean))].map(t => ({ text: t, value: t })))
const threadTypeFilters = computed(() => [...new Set(tableData.value.map(i => i.threadType).filter(Boolean))].map(t => ({ text: t, value: t })))
const platingFilters = computed(() => [...new Set(tableData.value.map(i => i.plating).filter(Boolean))].map(t => ({ text: t, value: t })))
function filterHandler(value: string, row: any, column: any) { return row[column.property] === value }

const form = ref<any>({
  id: '', customer: '', externalId: '', name: '', headType: '',
  punch: [], threadType: '', die: [], headSize: '', headHeight: '',
  length: '', threadDiameter: '', shankLength: '', wireMaterial: '', plating: '', remark: ''
})

// 解析关联表：punchId/dieId 是信息表的 ID，需要解析成名字
function resolveLinkNames(linkIdField: string, links: any[], infoList: any[]): Record<string, string[]> {
  const map: Record<string, string[]> = {}
  for (const link of links) {
    const specId = link.screwSpecId
    const itemId = link[linkIdField]
    if (!specId || !itemId) continue
    const info = infoList.find((i: any) => i.id === itemId)
    const name = info ? info.name : itemId
    if (!map[specId]) map[specId] = []
    if (!map[specId].includes(name)) map[specId].push(name)
  }
  return map
}

// 根据名字在信息表中查找 ID（多条同名的都返回）
function findIdsByNames(names: string[], infoList: any[]): string[] {
  const ids: string[] = []
  const seen = new Set<string>()
  for (const n of names) {
    for (const item of infoList) {
      if (item.name === n && !seen.has(item.id)) {
        ids.push(item.id)
        seen.add(item.id)
      }
    }
  }
  return ids
}

async function loadData() {
  loading.value = true
  try {
    const [screws, punches, dies, punchLinks, dieLinks] = await Promise.all([
      screwSpecApi.getAll(), punchApi.getAll(), dieApi.getAll(),
      punchLinkApi.getAll(), dieLinkApi.getAll()
    ])
    const punchNameMap = resolveLinkNames('punchId', punchLinks, punches)
    const dieNameMap = resolveLinkNames('dieId', dieLinks, dies)
    // 兼容旧数据：如果关联表没有数据，用主表的逗号分隔字段
    tableData.value = screws.map((s: any) => ({
      ...s,
      _punchNames: punchNameMap[s.id] || parseNames(s.punch),
      _dieNames: dieNameMap[s.id] || parseNames(s.die)
    }))
    punchList.value = punches
    dieList.value = dies
  } catch (error) { ElMessage.error('加载数据失败'); console.error(error) }
  finally { loading.value = false }
}

// ====== 冲头关联弹窗 ======
function showPunchDialog(row: any) {
  punchDialogRow.value = row
  punchDialogPrimary.value = row.punch || ''
  const names = row._punchNames || []
  const seen = new Set<string>()
  const items: any[] = []
  for (const n of names) {
    for (const m of punchList.value.filter(p => p.name === n)) {
      if (!seen.has(m.id)) { seen.add(m.id); items.push({ ...m }) }
    }
  }
  if (items.length === 0) for (const n of names) items.push({ id: n, name: n, spec: '', material: '' })
  stockCalcApi.calculate('punch').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.id === item.id || s.name === item.name)
      if (match) { item.currentStock = match.currentStock; item.safetyStock = match.safetyStock; item.status = match.status }
    }
    punchDialogItems.value = [...items]
  }).catch(() => { punchDialogItems.value = items })
  punchDialogVisible.value = true
}

async function setPunchPrimary(item: any) {
  if (item.name === punchDialogPrimary.value) return
  try {
    await screwSpecApi.update(punchDialogRow.value.id, { punch: item.name })
    punchDialogVisible.value = false
    loadData()
  } catch { ElMessage.error('设置失败') }
}

// ====== 牙板关联弹窗 ======
function showDieDialog(row: any) {
  dieDialogRow.value = row
  dieDialogPrimary.value = row.die || ''
  const names = row._dieNames || []
  const seen = new Set<string>()
  const items: any[] = []
  for (const n of names) {
    for (const m of dieList.value.filter(d => d.name === n)) {
      if (!seen.has(m.id)) { seen.add(m.id); items.push({ ...m }) }
    }
  }
  if (items.length === 0) for (const n of names) items.push({ id: n, name: n, machineType: '', wireDiameter: '' })
  stockCalcApi.calculate('die').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.id === item.id || s.name === item.name)
      if (match) { item.currentStock = match.currentStock; item.safetyStock = match.safetyStock; item.status = match.status }
    }
    dieDialogItems.value = [...items]
  }).catch(() => { dieDialogItems.value = items })
  dieDialogVisible.value = true
}

async function setDiePrimary(item: any) {
  if (item.name === dieDialogPrimary.value) return
  try {
    await screwSpecApi.update(dieDialogRow.value.id, { die: item.name })
    dieDialogVisible.value = false
    loadData()
  } catch { ElMessage.error('设置失败') }
}

// ====== CRUD ======
function handleAdd() {
  isEdit.value = false
  form.value = {
    id: '', customer: '', externalId: '', name: '', headType: '',
    punch: [], threadType: '', die: [], headSize: '', headHeight: '',
    length: '', threadDiameter: '', shankLength: '', wireMaterial: '', plating: '', remark: ''
  }
  dialogVisible.value = true
}

function handleEdit(row: any) {
  isEdit.value = true
  form.value = {
    ...row,
    punch: [...(row._punchNames || [])],
    die: [...(row._dieNames || [])]
  }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此规格？', '提示', { type: 'warning' })
    await screwSpecApi.remove(row.id)
    // 清理关联表
    const [pl, dl] = await Promise.all([punchLinkApi.getAll(), dieLinkApi.getAll()])
    for (const l of pl) { if (l.screwSpecId === row.id) await punchLinkApi.remove(l.id) }
    for (const l of dl) { if (l.screwSpecId === row.id) await dieLinkApi.remove(l.id) }
    ElMessage.success('删除成功')
    loadData()
  } catch (e) { if (e !== 'cancel') { ElMessage.error('删除失败'); console.error(e) } }
}

// 同步关联表：先删旧的，再建新的
async function syncLinks(screwSpecId: string, nameField: string, names: string[], linkApi: any, infoList: any[]) {
  const allLinks = await linkApi.getAll()
  for (const l of allLinks.filter((l: any) => l.screwSpecId === screwSpecId)) {
    await linkApi.remove(l.id)
  }
  const ids = findIdsByNames(names, infoList)
  for (const id of ids) {
    await linkApi.add({ [nameField]: id, screwSpecId })
  }
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      const punchNames = Array.isArray(form.value.punch) ? form.value.punch : []
      const dieNames = Array.isArray(form.value.die) ? form.value.die : []
      // 主表只存外显的第一个名字
      const payload = {
        ...form.value,
        punch: punchNames[0] || '',
        die: dieNames[0] || ''
      }
      let specId: string
      if (isEdit.value) {
        await screwSpecApi.update(form.value.id, payload)
        specId = form.value.id
      } else {
        const result = await screwSpecApi.add(payload)
        specId = result.id
      }
      // 同步关联表（用信息表的 ID）
      await syncLinks(specId, 'punchId', punchNames, punchLinkApi, punchList.value)
      await syncLinks(specId, 'dieId', dieNames, dieLinkApi, dieList.value)
      dialogVisible.value = false
      loadData()
    } catch (error) { ElMessage.error(isEdit.value ? '更新失败' : '添加失败'); console.error(error) }
  })
}
</script>

<style scoped>
.page-container { height: 100%; }
.page-container.is-fullscreen { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2000; background: #f0f2f5; padding: 20px; overflow: auto; }
.page-container.is-fullscreen .el-card { height: 100%; display: flex; flex-direction: column; margin: 0; }
.page-container.is-fullscreen .el-card__body { flex: 1; overflow: auto; }
.header-right { display: flex; gap: 8px; margin-left: auto; }
</style>
