<template>
  <el-dialog
    v-model="visible"
    class="attachment-workspace-dialog"
    :width="mode === 'edit' ? 'min(96vw, 1480px)' : 'min(94vw, 1320px)'"
    top="3vh"
    destroy-on-close
    append-to-body
    :before-close="handleBeforeClose"
    @closed="resetWorkspace"
  >
    <template #header>
      <div class="workspace-header">
        <div class="workspace-title-mark"><el-icon><Paperclip /></el-icon></div>
        <div class="workspace-heading">
          <strong>{{ mode === 'edit' ? '附件编辑与标注' : '附件预览' }}</strong>
          <span>{{ screwName || '螺丝规格' }} · {{ attachments.length }} 个附件</span>
        </div>
        <el-tag v-if="currentAttachment" :type="isPdf ? 'danger' : 'success'" effect="plain" round>
          {{ isPdf ? 'PDF 文档' : '图片' }}
        </el-tag>
      </div>
    </template>

    <div class="workspace-layout" :class="{ 'is-preview': mode === 'preview' }">
      <aside class="attachment-sidebar">
        <div class="sidebar-header">
          <div><strong>附件</strong><span>图片或 PDF，单个不超过 50MB</span></div>
          <el-button v-if="mode === 'edit'" type="primary" size="small" :loading="importing" @click="selectFiles">
            <el-icon><Plus /></el-icon>添加
          </el-button>
          <input
            ref="browserFileInputRef"
            class="browser-file-input"
            type="file"
            multiple
            accept=".png,.jpg,.jpeg,.webp,.gif,.pdf,image/png,image/jpeg,image/webp,image/gif,application/pdf"
            @change="handleBrowserFiles"
          />
        </div>

        <div v-if="loadingList" class="attachment-loading"><el-skeleton :rows="5" animated /></div>
        <el-empty v-else-if="!attachments.length" :description="mode === 'edit' ? '暂无附件，点击右上角添加' : '这条规格暂无附件'" :image-size="72" />
        <div v-else class="attachment-list">
          <button
            v-for="item in attachments"
            :key="item.id"
            type="button"
            class="attachment-item"
            :class="{ 'is-active': item.id === currentAttachment?.id }"
            @click="selectAttachment(item)"
          >
            <span class="attachment-type" :class="item.mimeType === 'application/pdf' ? 'is-pdf' : 'is-image'">
              <el-icon><Document v-if="item.mimeType === 'application/pdf'" /><Picture v-else /></el-icon>
            </span>
            <span class="attachment-meta">
              <strong>{{ item.displayName }}</strong>
              <span>{{ formatBytes(item.size) }} · {{ item.annotations.length }} 个标注</span>
            </span>
            <el-icon><ArrowRight /></el-icon>
          </button>
        </div>

        <div v-if="mode === 'edit' && currentAttachment" class="attachment-properties">
          <span class="section-label">附件信息</span>
          <el-input v-model="displayName" placeholder="附件名称" @change="saveDisplayName" />
          <div class="property-row"><span>原文件名</span><strong :title="currentAttachment.fileName">{{ currentAttachment.fileName }}</strong></div>
          <div class="property-row"><span>添加时间</span><strong>{{ currentAttachment.createdAt }}</strong></div>
          <el-button type="danger" plain :disabled="saving" @click="removeCurrent"><el-icon><Delete /></el-icon>删除附件</el-button>
        </div>
      </aside>

      <main class="attachment-main">
        <div v-if="!currentAttachment" class="workspace-empty">
          <div class="empty-mark"><el-icon><Files /></el-icon></div>
          <strong>选择附件开始{{ mode === 'edit' ? '标注' : '查看' }}</strong>
          <span>支持 PNG、JPG、WEBP、GIF 和 PDF</span>
        </div>
        <AttachmentCanvas
          v-else
          ref="attachmentCanvasRef"
          v-model="annotations"
          :content="currentContent"
          :readonly="mode === 'preview'"
          @change="markDirty"
        />
      </main>
    </div>

    <template #footer>
      <div class="workspace-footer">
        <span v-if="mode === 'edit'" class="save-state" :class="{ 'is-dirty': annotationsDirty }">
          <el-icon><CircleCheck v-if="!annotationsDirty" /><EditPen v-else /></el-icon>
          {{ annotationsDirty ? '标注有未保存修改' : '所有修改已保存' }}
        </span>
        <span v-else class="preview-tip">预览模式不会修改原附件</span>
        <div class="footer-actions">
          <el-button v-if="currentAttachment" :loading="exporting" @click="exportAnnotatedCopy">
            <el-icon><Download /></el-icon>导出标注副本
          </el-button>
          <el-button @click="visible = false">关闭</el-button>
          <el-button v-if="mode === 'edit'" type="primary" :loading="saving" :disabled="!annotationsDirty" @click="saveAnnotations">
            保存标注
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { ElMessage, ElMessageBox } from 'element-plus'
import AttachmentCanvas from './AttachmentCanvas.vue'
import { isTauriEnvironment, screwAttachmentApi, type AttachmentAnnotation, type AttachmentContent, type ScrewAttachment } from '../api'
import { isUserCancellation, showDetailedError } from '../utils/errorFeedback'

const props = withDefaults(defineProps<{
  modelValue: boolean
  screwSpecId: string
  screwName?: string
  mode?: 'preview' | 'edit'
}>(), { screwName: '', mode: 'preview' })

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  changed: [count: number]
}>()

const visible = computed({ get: () => props.modelValue, set: value => emit('update:modelValue', value) })
const attachments = ref<ScrewAttachment[]>([])
const currentAttachment = ref<ScrewAttachment | null>(null)
const browserFileInputRef = ref<HTMLInputElement | null>(null)
const currentContent = ref<AttachmentContent | null>(null)
const attachmentCanvasRef = ref<InstanceType<typeof AttachmentCanvas> | null>(null)
const annotations = ref<AttachmentAnnotation[]>([])
const displayName = ref('')
const loadingList = ref(false)
const importing = ref(false)
const saving = ref(false)
const exporting = ref(false)
const annotationsDirty = ref(false)
const isPdf = computed(() => currentAttachment.value?.mimeType === 'application/pdf')

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

async function loadList(preferredId = '') {
  if (!props.screwSpecId) return
  loadingList.value = true
  try {
    attachments.value = await screwAttachmentApi.list(props.screwSpecId)
    emit('changed', attachments.value.length)
    const target = attachments.value.find(item => item.id === preferredId)
      || attachments.value.find(item => item.id === currentAttachment.value?.id)
      || attachments.value[0]
    if (target) await selectAttachment(target)
    else resetSelection()
  } catch (error) {
    showDetailedError('加载螺丝规格附件列表', error)
  } finally {
    loadingList.value = false
  }
}

async function selectAttachment(item: ScrewAttachment) {
  if (annotationsDirty.value && currentAttachment.value?.id !== item.id) {
    try {
      await ElMessageBox.confirm('当前标注尚未保存，是否先保存再切换？', '未保存的标注', { confirmButtonText: '保存并切换', cancelButtonText: '放弃修改', distinguishCancelAndClose: true })
      await saveAnnotations()
    } catch (action) {
      if (action === 'close') return
    }
  }
  currentAttachment.value = item
  displayName.value = item.displayName
  annotations.value = item.annotations.map(annotation => ({ ...annotation }))
  annotationsDirty.value = false
  currentContent.value = null
  try {
    currentContent.value = await screwAttachmentApi.read(item.id)
  } catch (error) {
    showDetailedError('读取螺丝规格附件', error)
  }
}

async function selectFiles() {
  if (!props.screwSpecId) return
  if (!isTauriEnvironment()) {
    browserFileInputRef.value?.click()
    return
  }
  const result = await open({
    multiple: true,
    directory: false,
    filters: [
      { name: '图片和 PDF', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'pdf'] },
    ],
  })
  const files = Array.isArray(result) ? result : result ? [result] : []
  if (!files.length) return
  importing.value = true
  let latestId = ''
  try {
    for (const path of files) {
      const item = await screwAttachmentApi.import(props.screwSpecId, path)
      latestId = item.id
    }
    await loadList(latestId)
    ElMessage.success(`已添加 ${files.length} 个附件`)
  } catch (error) {
    showDetailedError('添加螺丝规格附件', error)
  } finally {
    importing.value = false
  }
}

async function handleBrowserFiles(event: Event) {
  const input = event.target as HTMLInputElement
  const files = [...(input.files || [])]
  input.value = ''
  if (!files.length) return
  importing.value = true
  let latestId = ''
  try {
    for (const file of files) {
      const item = await screwAttachmentApi.importFile(props.screwSpecId, file)
      latestId = item.id
    }
    await loadList(latestId)
    ElMessage.success(`已添加 ${files.length} 个附件`)
  } catch (error) {
    showDetailedError('添加螺丝规格附件', error)
  } finally {
    importing.value = false
  }
}

async function saveDisplayName() {
  if (!currentAttachment.value || !displayName.value.trim()) return
  saving.value = true
  try {
    const updated = await screwAttachmentApi.update(currentAttachment.value.id, { displayName: displayName.value })
    Object.assign(currentAttachment.value, updated)
    const listItem = attachments.value.find(item => item.id === updated.id)
    if (listItem) Object.assign(listItem, updated)
  } catch (error) {
    showDetailedError('重命名螺丝规格附件', error)
  } finally { saving.value = false }
}

function markDirty() { annotationsDirty.value = true }

async function saveAnnotations() {
  if (!currentAttachment.value) return
  saving.value = true
  try {
    const updated = await screwAttachmentApi.update(currentAttachment.value.id, { annotations: annotations.value })
    Object.assign(currentAttachment.value, updated)
    const listItem = attachments.value.find(item => item.id === updated.id)
    if (listItem) Object.assign(listItem, updated)
    annotationsDirty.value = false
    ElMessage.success('标注已保存')
  } catch (error) {
    showDetailedError('保存附件标注', error)
  } finally { saving.value = false }
}

function safeExportName(value: string) {
  return value.replace(/[\\/:*?"<>|]+/g, '_').replace(/\.[^.]+$/, '').trim() || '附件'
}

async function exportAnnotatedCopy() {
  if (!currentAttachment.value || !attachmentCanvasRef.value) return
  exporting.value = true
  try {
    const rendered = await attachmentCanvasRef.value.exportRenderedPng()
    const suffix = rendered.isPdf ? `-第${rendered.pageNumber}页-标注` : '-标注'
    const filename = `${safeExportName(currentAttachment.value.displayName)}${suffix}.png`
    if (isTauriEnvironment()) {
      const target = await save({ defaultPath: filename, filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
      if (!target) return
      await writeFile(target, rendered.bytes)
    } else {
      const blob = new Blob([rendered.bytes as BlobPart], { type: 'image/png' })
      const url = URL.createObjectURL(blob)
      const anchor = document.createElement('a')
      anchor.href = url
      anchor.download = filename
      anchor.click()
      URL.revokeObjectURL(url)
    }
    ElMessage.success(rendered.isPdf ? '已导出当前 PDF 页的标注副本' : '已导出标注副本')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('导出附件标注副本', error)
  } finally {
    exporting.value = false
  }
}

async function removeCurrent() {
  if (!currentAttachment.value) return
  try {
    await ElMessageBox.confirm(`确定删除附件“${currentAttachment.value.displayName}”吗？此操作无法撤销。`, '删除附件', { type: 'warning' })
    const id = currentAttachment.value.id
    await screwAttachmentApi.remove(id)
    await loadList()
    ElMessage.success('附件已删除')
  } catch (error) {
    if (!isUserCancellation(error)) showDetailedError('删除螺丝规格附件', error)
  }
}

async function handleBeforeClose(done: () => void) {
  if (props.mode !== 'edit' || !annotationsDirty.value) {
    done()
    return
  }
  try {
    await ElMessageBox.confirm('当前标注尚未保存，是否保存后关闭？', '未保存的标注', {
      confirmButtonText: '保存并关闭',
      cancelButtonText: '放弃修改',
      distinguishCancelAndClose: true,
      type: 'warning',
    })
    await saveAnnotations()
    if (!annotationsDirty.value) done()
  } catch (action) {
    if (action === 'cancel') done()
  }
}

function resetSelection() {
  currentAttachment.value = null
  currentContent.value = null
  annotations.value = []
  annotationsDirty.value = false
  displayName.value = ''
}

function resetWorkspace() {
  attachments.value = []
  resetSelection()
}

watch(() => [props.modelValue, props.screwSpecId] as const, ([opened, id]) => {
  if (opened && id) loadList()
}, { immediate: true })
</script>

<style scoped>
.workspace-header { display: flex; align-items: center; gap: 12px; padding-right: 36px; }
.workspace-title-mark { width: 40px; height: 40px; display: grid; place-items: center; flex-shrink: 0; border-radius: 12px; color: white; background: var(--primary); box-shadow: 0 8px 18px color-mix(in srgb, var(--primary) 26%, transparent); }
.workspace-title-mark .el-icon { font-size: 20px; }
.workspace-heading { min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.workspace-heading strong { color: var(--text-primary); font-size: 17px; }
.workspace-heading span { overflow: hidden; color: var(--text-muted); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.workspace-header > .el-tag { margin-left: auto; }
.workspace-layout { height: calc(88vh - 132px); min-height: 560px; display: grid; grid-template-columns: 300px minmax(0, 1fr); overflow: hidden; border: 1px solid var(--border); border-radius: 14px; background: var(--card-bg); }
.workspace-layout.is-preview { grid-template-columns: 260px minmax(0, 1fr); }
.attachment-sidebar { min-height: 0; display: flex; flex-direction: column; border-right: 1px solid var(--border); background: var(--surface-muted); }
.sidebar-header { min-height: 68px; padding: 14px; display: flex; align-items: center; justify-content: space-between; gap: 10px; border-bottom: 1px solid var(--border); }
.sidebar-header > div { min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.sidebar-header strong { color: var(--text-primary); font-size: 14px; }
.sidebar-header span { color: var(--text-muted); font-size: 11px; }
.browser-file-input { display: none; }
.attachment-loading, .attachment-list { flex: 1; min-height: 0; padding: 10px; overflow: auto; }
.attachment-list { display: flex; flex-direction: column; gap: 6px; }
.attachment-item { width: 100%; padding: 10px; display: grid; grid-template-columns: 38px minmax(0, 1fr) 16px; align-items: center; gap: 9px; border: 1px solid transparent; border-radius: 10px; color: var(--text-secondary); background: transparent; text-align: left; cursor: pointer; transition: 160ms ease; }
.attachment-item:hover { border-color: var(--border); background: var(--card-bg); }
.attachment-item.is-active { border-color: color-mix(in srgb, var(--primary) 35%, var(--border)); background: var(--card-bg); box-shadow: 0 6px 14px rgba(15, 23, 42, 0.06); }
.attachment-type { width: 38px; height: 38px; display: grid; place-items: center; border-radius: 9px; }
.attachment-type.is-image { color: #2f855a; background: color-mix(in srgb, #48bb78 13%, var(--card-bg)); }
.attachment-type.is-pdf { color: #c53030; background: color-mix(in srgb, #e53e3e 12%, var(--card-bg)); }
.attachment-meta { min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.attachment-meta strong { overflow: hidden; color: var(--text-primary); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.attachment-meta span { color: var(--text-muted); font-size: 10px; }
.attachment-properties { padding: 14px; display: flex; flex-direction: column; gap: 10px; border-top: 1px solid var(--border); background: var(--card-bg); }
.section-label { color: var(--text-muted); font-size: 11px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; }
.property-row { display: grid; grid-template-columns: 66px minmax(0, 1fr); gap: 8px; color: var(--text-muted); font-size: 11px; }
.property-row strong { overflow: hidden; color: var(--text-secondary); font-weight: 500; text-overflow: ellipsis; white-space: nowrap; }
.attachment-main { min-width: 0; min-height: 0; overflow: hidden; background: var(--bg); }
.workspace-empty { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--text-muted); }
.workspace-empty strong { color: var(--text-primary); font-size: 16px; }
.workspace-empty span { font-size: 12px; }
.empty-mark { width: 72px; height: 72px; margin-bottom: 4px; display: grid; place-items: center; border: 1px solid var(--border); border-radius: 22px; background: var(--card-bg); box-shadow: var(--shadow-card); }
.empty-mark .el-icon { color: var(--primary); font-size: 30px; }
.workspace-footer { width: 100%; display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.save-state, .preview-tip { display: flex; align-items: center; gap: 6px; color: var(--text-muted); font-size: 12px; }
.save-state.is-dirty { color: var(--el-color-warning); }
.footer-actions { display: flex; gap: 8px; }
@media (max-width: 900px) { .workspace-layout, .workspace-layout.is-preview { grid-template-columns: 220px minmax(600px, 1fr); overflow: auto; } }
</style>

<style>
.attachment-workspace-dialog { border-radius: 18px; overflow: hidden; background: var(--card-bg); }
.attachment-workspace-dialog .el-dialog__header { margin: 0; padding: 16px 20px; border-bottom: 1px solid var(--border); }
.attachment-workspace-dialog .el-dialog__body { padding: 14px 18px; }
.attachment-workspace-dialog .el-dialog__footer { padding: 12px 18px; border-top: 1px solid var(--border); }
</style>
