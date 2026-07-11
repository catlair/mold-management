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

      <div class="table-wrapper" :style="{ height: isFullscreen ? 'calc(100vh - 60px)' : 'calc(100vh - 140px)' }">
      <vxe-table
        ref="vxeTableRef"
        :data="tableData"
        :scroll-x="{ enabled: true, gt: 0 }"
        border
        :loading="loading"
        :show-overflow="true"
      >
        <vxe-column field="name" title="螺丝名称" width="160" sortable />
        <vxe-column field="headType" title="头型" width="120" sortable />
        <vxe-column field="punch" title="冲头" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.punch" type="primary" :underline="false" @click="showPunchDialog(row)">{{ row.punch }}</el-link>
            <span v-else>-</span>
          </template>
        </vxe-column>
        <vxe-column field="threadType" title="牙型" width="120" sortable />
        <vxe-column field="die" title="牙板" width="120" sortable>
          <template #default="{ row }">
            <el-link v-if="row.die" type="success" :underline="false" @click="showDieDialog(row)">{{ row.die }}</el-link>
            <span v-else>-</span>
          </template>
        </vxe-column>
        <vxe-column field="headSize" title="头/垫片大小" width="140" sortable />
        <vxe-column field="headHeight" title="头高" width="100" sortable />
        <vxe-column field="length" title="长度" width="100" sortable />
        <vxe-column field="threadDiameter" title="牙径" width="100" sortable />
        <vxe-column field="shankLength" title="光钉长度" width="120" sortable />
        <vxe-column field="wireMaterial" title="线材" width="100" sortable />
        <vxe-column field="plating" title="电镀" width="120" sortable />
        <vxe-column field="customer" title="客户名" width="120" sortable />
        <vxe-column field="externalId" title="外部ID" width="120" sortable />
        <vxe-column field="remark" title="备注" min-width="140" />
        <vxe-column title="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="handleEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" v-if="allowDelete" @click="handleDelete(row)">删除</el-button>
          </template>
        </vxe-column>
      </vxe-table>
      </div>

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
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSubmit">确定</el-button>
        </span>
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
import { useAllowDelete } from '../composables/useAllowDelete'
import { useHighlight } from '../composables/useHighlight'
import { toShortCode, matchPunchNames } from '../utils/punchName'

const { allowDelete } = useAllowDelete()

const tableData = ref<any[]>([])
useHighlight(tableData)
const punchList = ref<any[]>([])
const dieList = ref<any[]>([])
const dialogVisible = ref(false)
const isEdit = ref(false)
const isFullscreen = ref(false)
const loading = ref(true)

// 冲头关联弹窗
const punchDialogVisible = ref(false)
const punchDialogItems = ref<any[]>([])
const punchDialogPrimary = ref('')
const punchDialogRow = ref<any>({})

// 牙板关联弹窗
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

// 冲头选项
const punchOptions = computed(() => {
  const names = [...new Set(punchList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => {
    const short = toShortCode(name)
    const display = short || name
    const fullName = short ? name : ''
    return {
      name: display,
      label: fullName ? `${display} (${fullName})` : display,
      specs: punchList.value.filter(p => p.name === name).map(p => `${p.spec}${p.material ? '(' + p.material + ')' : ''}`).join('、')
    }
  })
})

// 牙板选项
const dieOptions = computed(() => {
  const names = [...new Set(dieList.value.map(item => item.name).filter(Boolean))]
  return names.map(name => ({
    name,
    specs: dieList.value.filter(d => d.name === name).map(d => `${d.machineType}${d.wireDiameter ? '(' + d.wireDiameter + ')' : ''}`).join('、')
  }))
})

const form = ref<any>({
  id: '', customer: '', externalId: '', name: '', headType: '',
  punch: [], threadType: '', die: [], headSize: '', headHeight: '',
  length: '', threadDiameter: '', shankLength: '', wireMaterial: '', plating: '', remark: ''
})

// 关联表解析
function resolveLinks(linkIdField: string, links: any[], infoList: any[]) {
  const map: Record<string, { ids: string[]; names: string[] }> = {}
  for (const link of links) {
    const specId = link.screwSpecId
    const itemId = link[linkIdField]
    if (!specId || !itemId) continue
    if (!map[specId]) map[specId] = { ids: [], names: [] }
    if (!map[specId].ids.includes(itemId)) {
      map[specId].ids.push(itemId)
      const info = infoList.find((i: any) => i.id === itemId)
      const shortName = toShortCode(info?.name || '') || info?.name || itemId
      map[specId].names.push(shortName)
    }
  }
  return map
}

function findIdsByNames(names: string[], infoList: any[]) {
  const ids: string[] = []
  const seen = new Set<string>()
  for (const n of names) {
    for (const item of infoList) {
      if (matchPunchNames(n, item.name) && !seen.has(item.id)) {
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
    const punchMap = resolveLinks('punchId', punchLinks, punches)
    const dieMap = resolveLinks('dieId', dieLinks, dies)
    tableData.value = screws.map((s: any) => ({
      ...s,
      _punchIds: punchMap[s.id]?.ids || [],
      _punchNames: punchMap[s.id]?.names || [],
      _dieIds: dieMap[s.id]?.ids || [],
      _dieNames: dieMap[s.id]?.names || []
    }))
    punchList.value = punches
    dieList.value = dies
  } catch (error) { ElMessage.error('加载数据失败'); console.error(error) }
  finally { loading.value = false }
}

// 冲头关联弹窗
function showPunchDialog(row: any) {
  punchDialogRow.value = row
  punchDialogPrimary.value = row.punch || ''
  const ids = row._punchIds || []
  const items = ids.map((id: string) => punchList.value.find(p => p.id === id)).filter(Boolean).map((p: any) => ({ ...p }))
  stockCalcApi.calculate('punch').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.punchId === item.id)
      if (match) { item.currentStock = match.currentStock; item.safetyStock = match.safetyStock; item.status = match.status }
    }
    punchDialogItems.value = [...items]
  }).catch(() => { punchDialogItems.value = items })
  punchDialogVisible.value = true
}

async function setPunchPrimary(item: any) {
  if (matchPunchNames(item.name, punchDialogPrimary.value)) return
  try {
    const shortName = toShortCode(item.name) || item.name
    await screwSpecApi.update(punchDialogRow.value.id, { punch: shortName })
    punchDialogVisible.value = false
    loadData()
  } catch { ElMessage.error('设置失败') }
}

// 牙板关联弹窗
function showDieDialog(row: any) {
  dieDialogRow.value = row
  dieDialogPrimary.value = row.die || ''
  const ids = row._dieIds || []
  const items = ids.map((id: string) => dieList.value.find(d => d.id === id)).filter(Boolean).map((d: any) => ({ ...d }))
  stockCalcApi.calculate('die').then((sd: any[]) => {
    for (const item of items) {
      const match = sd.find((s: any) => s.dieId === item.id)
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

// CRUD
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
  form.value = { ...row, punch: [...(row._punchNames || [])], die: [...(row._dieNames || [])] }
  dialogVisible.value = true
}

async function handleDelete(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此规格？', '提示', { type: 'warning' })
    await screwSpecApi.remove(row.id)
    const [pl, dl] = await Promise.all([punchLinkApi.getAll(), dieLinkApi.getAll()])
    for (const l of pl) { if (l.screwSpecId === row.id) await punchLinkApi.remove(l.id) }
    for (const l of dl) { if (l.screwSpecId === row.id) await dieLinkApi.remove(l.id) }
    ElMessage.success('删除成功')
    loadData()
  } catch (e) { if (e !== 'cancel') { ElMessage.error('删除失败'); console.error(e) } }
}

async function syncLinks(screwSpecId: string, nameField: string, names: string[], linkApi: any, infoList: any[]) {
  const allLinks = await linkApi.getAll()
  for (const l of allLinks.filter((l: any) => l.screwSpecId === screwSpecId)) { await linkApi.remove(l.id) }
  const ids = findIdsByNames(names, infoList)
  for (const id of ids) { await linkApi.add({ [nameField]: id, screwSpecId }) }
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    try {
      const punchNames = Array.isArray(form.value.punch) ? form.value.punch : []
      const dieNames = Array.isArray(form.value.die) ? form.value.die : []
      const payload = { ...form.value, punch: punchNames[0] || '', die: dieNames[0] || '' }
      let specId: string
      if (isEdit.value) {
        await screwSpecApi.update(form.value.id, payload)
        specId = form.value.id
      } else {
        const result = await screwSpecApi.add(payload)
        specId = result.id
      }
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
.page-container.is-fullscreen { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 2000; background: #fff; padding: 0; overflow: auto; }
.page-container.is-fullscreen .el-card { height: 100%; display: flex; flex-direction: column; margin: 0; border: none; border-radius: 0; box-shadow: none; }
.page-container.is-fullscreen .el-card__header { display: none; }
.page-container.is-fullscreen .el-card__body { flex: 1; overflow: auto; padding: 12px; }
.table-wrapper { overflow: auto; }
.header-right { display: flex; gap: 8px; margin-left: auto; }
</style>
