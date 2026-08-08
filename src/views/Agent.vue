<template>
  <div class="agent-page">
    <div class="agent-header">
      <div>
        <div class="eyebrow">SYSTEM AGENT</div>
        <h1><el-icon><MagicStick /></el-icon> AI 助手</h1>
        <p>用自然语言查找模具、库存与出入库记录；涉及系统变更时，先预览，再由你确认执行。</p>
      </div>
      <div class="agent-header-actions">
        <el-button plain :disabled="loading" @click="uploadExcel"><el-icon><Upload /></el-icon> 上传 Excel 分析</el-button>
        <el-button plain @click="configVisible = true"><el-icon><Setting /></el-icon> API 配置</el-button>
      </div>
    </div>

    <div class="agent-grid has-sidebar">
      <el-card class="session-sidebar" shadow="never">
        <div class="session-toolbar">
          <el-button type="primary" size="small" class="new-session-btn" @click="newSession"><el-icon><Plus /></el-icon> 新对话</el-button>
        </div>
        <div class="session-list">
          <div v-for="s in sortedSessions" :key="s.id" class="session-item" :class="{ 'is-active': s.id === activeSessionId }" @click="switchSession(s.id)">
            <template v-if="renamingId === s.id">
              <el-input v-model="renameValue" size="small" @click.stop @keydown.enter="finishRename" @blur="finishRename" />
            </template>
            <template v-else>
              <span class="session-title" :title="s.title" @dblclick.stop="startRename(s)">{{ s.title }}</span>
              <el-button size="small" text type="danger" class="session-del" @click.stop="removeSession(s.id)"><el-icon><Delete /></el-icon></el-button>
            </template>
          </div>
          <div v-if="sessions.length === 0" class="session-empty">暂无对话</div>
        </div>
        <div class="session-hint">双击标题可重命名</div>
      </el-card>
      <el-card class="chat-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span><el-icon><ChatDotRound /></el-icon> 对话</span>
            <div class="card-header-actions">
              <el-tag v-if="pageContext" size="small" type="info" effect="plain" closable @close="pageContext = ''">
                <el-icon><Pointer /></el-icon> {{ pageContext }}
              </el-tag>
              <el-tag size="small" :type="currentProfile && (currentProfile.apiKeyRequired === false || currentProfile.apiKeyConfigured) ? 'success' : 'warning'">
                {{ currentProfile ? `${profileLabel(currentProfile)} · ${currentProfile.model || '默认模型'}${currentProfile.apiKeyRequired === false ? ' · 免费' : ''}` : '未配置 AI' }}
              </el-tag>
              <el-button v-if="messages.length > 0" size="small" text type="danger" @click="clearChat"><el-icon><Delete /></el-icon> 清空对话</el-button>
            </div>
          </div>
        </template>
        <div ref="messagesRef" class="messages">
          <div v-if="messages.length === 0" class="empty-chat">
            <el-icon :size="42"><MagicStick /></el-icon>
            <strong>从一个问题开始</strong>
            <span>例如：找出库存低于安全库存的冲头</span>
            <div class="suggestions">
              <el-button v-for="item in suggestions" :key="item" size="small" plain @click="askSuggestion(item)">{{ item }}</el-button>
            </div>
          </div>
          <div v-for="message in messages" :key="message.id" class="message-row" :class="message.role">
            <div class="message-avatar"><el-icon><User v-if="message.role === 'user'" /><MagicStick v-else /></el-icon></div>
            <div class="message-bubble">
              <details v-if="message.reasoning" class="reasoning-block">
                <summary><el-icon><MagicStick /></el-icon> 思考过程</summary>
                <div class="reasoning-text">{{ message.reasoning }}</div>
              </details>
              <div class="message-text">{{ message.content }}</div>
              <div v-for="(change, index) in message.changes" :key="`${message.id}-${index}`" class="change-card">
                <div class="change-title"><el-icon><EditPen /></el-icon> 检测到待确认变更</div>
                <el-descriptions :column="1" size="small" border>
                  <el-descriptions-item label="操作">{{ operationLabel(change.operation) }}</el-descriptions-item>
                  <el-descriptions-item label="数据表">{{ tableLabel(change) }}</el-descriptions-item>
                  <el-descriptions-item v-if="change.id" label="记录 ID">{{ change.id }}</el-descriptions-item>
                  <el-descriptions-item label="字段变更">
                    <div v-for="(value, key) in change.fields" :key="key" class="field-change">
                      <span>{{ key }}</span><b>{{ String(value) }}</b>
                    </div>
                  </el-descriptions-item>
                </el-descriptions>
                <el-alert title="执行前会自动备份；删除还需要系统删除权限。" type="warning" :closable="false" show-icon />
                <div class="change-actions">
                  <el-button type="primary" size="small" :loading="applyingId === `${message.id}-${index}`" @click="applyChange(message, change, index)">确认并执行</el-button>
                  <el-button size="small" @click="discardChange(message, change, index)">取消</el-button>
                </div>
              </div>
              <div v-if="message.role === 'assistant'" class="message-actions">
                <el-button size="small" text type="primary" :disabled="loading" @click="retryMessage(message)"><el-icon><RefreshRight /></el-icon> 重新生成</el-button>
              </div>
            </div>
          </div>
          <div v-if="loading" class="message-row assistant">
            <div class="message-avatar"><el-icon><MagicStick /></el-icon></div>
            <div class="message-bubble">
              <div v-if="streamReasoning" class="reasoning-live">
                <div class="reasoning-live-title"><el-icon><MagicStick /></el-icon> 思考过程</div>
                <div class="reasoning-text">{{ streamReasoning }}</div>
              </div>
              <div v-if="streamText" class="stream-text">{{ streamText }}</div>
              <div v-else-if="!streamReasoning" class="loading-bubble">正在思考…</div>
            </div>
          </div>
        </div>
        <div class="composer">
          <el-input v-model="question" type="textarea" :rows="3" resize="none" maxlength="1000" show-word-limit placeholder="描述你要查找或更改的内容…" @keydown.enter.exact.prevent="send" />
          <div class="composer-footer"><span>Enter 发送 · AI 只会访问业务数据，不会读取 API Key</span><el-button type="primary" :loading="loading" :disabled="!question.trim()" @click="send"><el-icon><Promotion /></el-icon> 发送</el-button></div>
        </div>
      </el-card>
    </div>

    <el-dialog v-model="configVisible" title="AI 服务配置（可添加多份，自主切换）" width="720px" :close-on-click-modal="false">
      <div class="profile-list">
        <div v-for="p in profiles" :key="p.id" class="profile-item" :class="{ 'is-active': p.id === active }">
          <div class="profile-main">
            <span class="profile-name">{{ profileLabel(p) }}</span>
            <el-tag size="small" :type="p.kind === 'builtin' ? 'primary' : 'warning'" effect="plain">{{ p.kind === 'builtin' ? '内置' : '自定义' }}</el-tag>
            <el-tag v-if="p.id === active" size="small" type="success" effect="dark">当前启用</el-tag>
          </div>
          <div class="profile-meta">
            {{ p.kind === 'builtin' ? builtinLabel(p.provider) : (p.format === 'cc' ? 'Anthropic 格式' : 'OpenAI 格式') }} · {{ p.model || '默认模型' }} · {{ profileKeyText(p) }}
          </div>
          <div class="profile-actions">
            <el-button size="small" type="primary" plain :disabled="p.id === active" @click="activateProfile(p.id)">启用</el-button>
            <el-button size="small" @click="openEdit(p)">编辑</el-button>
            <el-button size="small" type="danger" plain :disabled="profiles.length <= 1" @click="removeProfile(p.id)">删除</el-button>
          </div>
        </div>
        <div v-if="profiles.length === 0" class="profile-empty">尚未添加任何 AI 配置，点击下方按钮添加。</div>
        <el-button type="primary" plain class="add-profile-btn" @click="openAdd"><el-icon><Plus /></el-icon> 添加配置</el-button>
      </div>

      <template #footer><el-button @click="configVisible = false">关闭</el-button></template>
    </el-dialog>

    <el-dialog v-model="editVisible" :title="editingId ? '编辑 AI 配置' : '添加 AI 配置'" width="560px" :close-on-click-modal="false">
      <el-form label-width="100px" class="config-form">
        <el-form-item label="配置名称"><el-input v-model="editForm.name" maxlength="40" placeholder="如：opencode Zen 主力（可留空自动命名）" /></el-form-item>
        <el-form-item label="配置类型">
          <el-radio-group v-model="editForm.kind" @change="onKindChange">
            <el-radio value="builtin">内置服务商</el-radio>
            <el-radio value="custom">自定义接口</el-radio>
          </el-radio-group>
        </el-form-item>
        <template v-if="editForm.kind === 'builtin'">
          <el-form-item label="服务商">
            <el-select v-model="editForm.provider" @change="onBuiltinChange">
              <el-option v-for="b in builtins" :key="b.value" :label="b.label" :value="b.value" />
            </el-select>
          </el-form-item>
          <el-form-item label="模型名称">
            <el-select v-if="editForm.provider.startsWith('opencode-zen')" v-model="editForm.model" filterable default-first-option placeholder="选择 OpenCode Zen 模型">
              <el-option v-for="m in currentZenModels" :key="m.id" :label="`${m.label} · ${m.id}`" :value="m.id">
                <span>{{ m.label }}</span>
                <span class="model-id">{{ m.id }}</span>
              </el-option>
            </el-select>
            <el-input v-else v-model="editForm.model" :placeholder="`默认 ${builtinDefaultModel(editForm.provider)}，可修改`" />
          </el-form-item>
          <el-alert
            v-if="!builtinNeedsKey(editForm.provider)"
            :title="`${builtinLabel(editForm.provider)} 为免费服务，无需 API Key，直接选择模型即可使用。`"
            type="success"
            :closable="false"
            show-icon
            class="provider-alert"
          />
          <el-alert v-else :title="`${builtinLabel(editForm.provider)} 已内置接口，只需填写 API Key。`" type="success" :closable="false" show-icon class="provider-alert" />
        </template>
        <template v-else>
          <el-form-item label="接口格式">
            <el-select v-model="editForm.format">
              <el-option label="OpenAI 兼容" value="openai" />
              <el-option label="Anthropic 兼容" value="cc" />
            </el-select>
          </el-form-item>
          <el-form-item label="API 地址"><el-input v-model="editForm.endpoint" placeholder="https://api.example.com/v1 或 …/v1/messages" /></el-form-item>
          <el-form-item label="模型名称"><el-input v-model="editForm.model" placeholder="服务端支持的模型代码" /></el-form-item>
        </template>
        <el-form-item v-if="editForm.kind === 'custom' || builtinNeedsKey(editForm.provider)" label="API Key"><el-input v-model="editForm.apiKey" type="password" show-password autocomplete="new-password" :placeholder="editingKeyConfigured ? '已配置，留空保持不变' : '请输入该配置的 API Key'" /></el-form-item>
        <el-form-item v-else label="API Key">
          <span class="free-key-hint">免费服务，无需 API Key</span>
        </el-form-item>
      </el-form>
      <template #footer><el-button @click="editVisible = false">取消</el-button><el-button type="primary" :loading="saving" @click="saveEdit">保存</el-button></template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'Agent' })
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { agentApi, type AgentBuiltinView, type AgentChange, type AgentChatResult, type AgentProfileInput, type AgentProfileView, type ZenModelView } from '../api'
import { ZEN_FREE_MODEL_PRESETS, ZEN_MODEL_PRESETS, normalizeZenModels } from '../config/zenModels'
import { useTheme } from '../composables/useTheme'

interface Message {
  id: number
  role: 'user' | 'assistant'
  content: string
  changes?: AgentChange[] | null
  reasoning?: string | null
  attachment?: { kind: 'excel'; path: string; name: string }
}
interface ChatSession { id: string; title: string; messages: Message[]; updatedAt: number }
const suggestions = ['哪些冲头库存低于安全库存？', '列出库存状态为「不足」的模具', '查询名称含 M3 的螺丝规格', '最近有哪些冲头入库？']
const messages = ref<Message[]>([])
// ===== 多会话管理 =====
const SESSIONS_STORAGE_KEY = 'mold-agent-sessions-v1'
const LEGACY_STORAGE_KEY = 'mold-agent-chat-v1'
// 仅用于区分「本次应用运行首次进入 AI」与「页面切换后返回 AI」。
// sessionStorage 在应用进程内保留、重启应用后清空。
const AI_VISITED_SESSION_KEY = 'mold-agent-visited-this-run'
const sessions = ref<ChatSession[]>([])
const activeSessionId = ref('')
const renamingId = ref('')
const renameValue = ref('')
// 会话列表按最近使用排序（最新置顶），与主流 AI 一致
const sortedSessions = computed(() => [...sessions.value].sort((a, b) => b.updatedAt - a.updatedAt))
const sanitizeMessage = (item: unknown): Message | null => {
  if (!item || typeof item !== 'object' || !('role' in item) || !('content' in item)) return null
  const role = (item as Message).role
  if (role !== 'user' && role !== 'assistant') return null
  if (typeof (item as Message).content !== 'string') return null
  const legacy = item as Message & { change?: AgentChange | null }
  if (!legacy.changes && legacy.change) legacy.changes = [legacy.change]
  delete legacy.change
  return legacy
}
const loadSessions = () => {
  try {
    const raw = localStorage.getItem(SESSIONS_STORAGE_KEY)
    if (raw) {
      const parsed: unknown = JSON.parse(raw)
      if (parsed && typeof parsed === 'object' && 'sessions' in parsed) {
        const data = parsed as { sessions: ChatSession[]; activeId?: string }
        sessions.value = (data.sessions ?? []).map(s => ({ ...s, messages: (s.messages ?? []).map(sanitizeMessage).filter(Boolean) as Message[] }))
        activeSessionId.value = data.activeId && sessions.value.some(s => s.id === data.activeId) ? data.activeId : (sessions.value[0]?.id ?? '')
      }
    }
    if (sessions.value.length === 0) {
      // 旧版单会话迁移
      try {
        const legacyRaw = localStorage.getItem(LEGACY_STORAGE_KEY)
        if (legacyRaw) {
          const parsed: unknown = JSON.parse(legacyRaw)
          if (Array.isArray(parsed)) {
            const msgs = parsed.map(sanitizeMessage).filter(Boolean) as Message[]
            if (msgs.length > 0) {
              sessions.value = [{ id: 'legacy-1', title: msgs[0].role === 'user' ? msgs[0].content.slice(0, 20) : '历史对话', messages: msgs, updatedAt: Date.now() }]
              activeSessionId.value = 'legacy-1'
            }
          }
        }
      } catch { /* 旧数据损坏则忽略 */ }
    }
    if (sessions.value.length === 0) {
      sessions.value = [{ id: 'session-1', title: '新对话', messages: [], updatedAt: Date.now() }]
    }
    const visitedThisRun = sessionStorage.getItem(AI_VISITED_SESSION_KEY) === '1'
    const savedActive = sessions.value.find(s => s.id === activeSessionId.value)
    if (visitedThisRun && savedActive) {
      // 应用内页面切换后回来：恢复刚才正在使用的会话，不创建/切换新会话。
      messages.value = [...savedActive.messages]
    } else {
      // 本次应用运行第一次进入 AI：默认进入空白新对话（复用已有空白，避免列表膨胀）。
      const blank = [...sessions.value].sort((a, b) => b.updatedAt - a.updatedAt).find(s => s.messages.length === 0)
      if (blank) {
        activeSessionId.value = blank.id
        messages.value = []
      } else {
        const id = `session-${Date.now()}`
        sessions.value.unshift({ id, title: '新对话', messages: [], updatedAt: Date.now() })
        activeSessionId.value = id
        messages.value = []
      }
      sessionStorage.setItem(AI_VISITED_SESSION_KEY, '1')
    }
    persistSessions()
  } catch {
    sessions.value = [{ id: 'session-1', title: '新对话', messages: [], updatedAt: Date.now() }]
    activeSessionId.value = 'session-1'
  }
}
const persistSessions = () => {
  try {
    localStorage.setItem(SESSIONS_STORAGE_KEY, JSON.stringify({ sessions: sessions.value, activeId: activeSessionId.value }))
  } catch { /* 忽略存储失败 */ }
}
const saveCurrentSession = () => {
  const session = sessions.value.find(s => s.id === activeSessionId.value)
  if (!session) return
  session.messages = messages.value
  session.updatedAt = Date.now()
  const firstUser = messages.value.find(m => m.role === 'user')
  if (firstUser && (session.title === '新对话' || !session.title)) session.title = firstUser.content.slice(0, 20)
}
const saveChat = () => { saveCurrentSession(); persistSessions() }
watch(messages, saveChat, { deep: true })
watch(messages, async () => { await scrollBottom() }, { deep: true })
const newSession = () => {
  saveCurrentSession()
  const id = `session-${Date.now()}`
  sessions.value.unshift({ id, title: '新对话', messages: [], updatedAt: Date.now() })
  activeSessionId.value = id
  messages.value = []
  persistSessions()
  streamReasoning.value = ''
  streamText.value = ''
  scrollBottom()
}
const switchSession = (id: string) => {
  if (id === activeSessionId.value) return
  saveCurrentSession()
  activeSessionId.value = id
  const target = sessions.value.find(s => s.id === id)
  if (target) target.updatedAt = Date.now()
  messages.value = target ? [...target.messages] : []
  persistSessions()
  scrollBottom()
}
const removeSession = async (id: string) => {
  if (sessions.value.length <= 1) return ElMessage.warning('至少保留一个对话')
  try {
    await ElMessageBox.confirm('删除该对话（不影响已保存的业务数据）。', '删除对话', { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' })
  } catch { return }
  sessions.value = sessions.value.filter(s => s.id !== id)
  if (activeSessionId.value === id) {
    const next = sessions.value[0]
    activeSessionId.value = next?.id ?? ''
    messages.value = next ? [...next.messages] : []
  }
  persistSessions()
}
const startRename = (session: ChatSession) => { renamingId.value = session.id; renameValue.value = session.title }
const finishRename = () => {
  const session = sessions.value.find(s => s.id === renamingId.value)
  if (session) { session.title = renameValue.value.trim() || '新对话'; persistSessions() }
  renamingId.value = ''
}
const question = ref('')
const loading = ref(false)
const applyingId = ref<string | null>(null)
const messagesRef = ref<HTMLElement>()
// 页面联动：从业务页面（如冲头管理）跳转进入时带上的当前页面上下文
const route = useRoute()
const pageContext = ref('')
const buildPageContext = () => {
  const from = String(route.query.from ?? '')
  const filter = String(route.query.filter ?? '')
  const tab = String(route.query.tab ?? '')
  if (!from && !filter) { pageContext.value = ''; return }
  const parts: string[] = []
  if (from) parts.push(`页面：${from}`)
  if (tab && tab !== 'info') parts.push(`页签：${tab}`)
  if (filter) parts.push(`当前筛选：${filter}`)
  pageContext.value = parts.join('，')
}
// 流式输出状态：等待 AI 响应期间实时展示思考过程与回答增量
const streamReasoning = ref('')
const streamText = ref('')
let unlistenStream: (() => void) | null = null
const configVisible = ref(false)
const saving = ref(false)
const profiles = ref<AgentProfileView[]>([])
const active = ref('')
const builtins = ref<AgentBuiltinView[]>([])
const zenFreeModels = ref<ZenModelView[]>([...ZEN_FREE_MODEL_PRESETS])
const zenModels = ref<ZenModelView[]>([...ZEN_MODEL_PRESETS])
const currentProfile = computed(() => profiles.value.find(p => p.id === active.value) ?? profiles.value[0] ?? null)

// 添加/编辑配置弹窗
const editVisible = ref(false)
const editingId = ref('')
const editingKeyConfigured = ref(false)
const editForm = reactive({ name: '', kind: 'builtin' as 'builtin' | 'custom', provider: 'opencode-zen', format: 'openai', endpoint: '', model: 'deepseek-v4-flash-free', apiKey: '' })
const pendingKeys = new Map<string, string>()

const builtinLabel = (value: string) => builtins.value.find(b => b.value === value)?.label ?? value
const builtinDefaultModel = (value: string) => builtins.value.find(b => b.value === value)?.model ?? ''
const currentZenModels = computed(() => editForm.provider === 'opencode-zen-free' ? zenFreeModels.value : zenModels.value)
/** 内置服务商是否需要 API Key（免费版 opencode Zen 为 false，与模型选择无关） */
const builtinNeedsKey = (value: string) => {
  const builtin = builtins.value.find(b => b.value === value)
  return builtin ? builtin.needsApiKey !== false : true
}
const profileKeyText = (p: AgentProfileView) => {
  if (p.apiKeyRequired === false) return '免费服务'
  return p.apiKeyConfigured ? 'Key：已配置' : 'Key：未配置'
}
const profileLabel = (p: AgentProfileView) => {
  if (p.name.trim()) return p.name.trim()
  if (p.kind === 'builtin') return builtinLabel(p.provider)
  return p.format === 'cc' ? 'Anthropic' : 'OpenAI'
}

const openAdd = () => {
  editingId.value = ''
  editingKeyConfigured.value = false
  const defaultProvider = 'opencode-zen-free'
  Object.assign(editForm, {
    name: builtinLabel(defaultProvider),
    kind: 'builtin',
    provider: defaultProvider,
    format: 'openai',
    endpoint: '',
    model: zenFreeModels.value[0]?.id || 'deepseek-v4-flash-free',
    apiKey: '',
  })
  editVisible.value = true
}
const openEdit = (p: AgentProfileView) => {
  editingId.value = p.id
  editingKeyConfigured.value = p.apiKeyConfigured
  Object.assign(editForm, { name: p.name, kind: p.kind, provider: p.provider, format: p.format, endpoint: p.endpoint, model: p.model, apiKey: '' })
  if (p.kind === 'builtin' && !builtinNeedsKey(p.provider)) {
    // 免费服务无需 Key：不显示 Key 输入，也不保留旧的 Key 状态
    editingKeyConfigured.value = false
  }
  editVisible.value = true
}
const onKindChange = () => {
  if (editForm.kind === 'builtin') {
    editForm.provider = 'opencode-zen-free'
    editForm.model = zenFreeModels.value[0]?.id || 'deepseek-v4-flash-free'
    editForm.format = 'openai'
    editForm.apiKey = ''
  } else {
    editForm.format = 'openai'
  }
}
const onBuiltinChange = () => {
  editForm.model = builtinDefaultModel(editForm.provider)
  if (!builtinNeedsKey(editForm.provider)) editForm.apiKey = ''
}

const saveEdit = async () => {
  const kind = editForm.kind
  const profile: AgentProfileInput = {
    id: editingId.value || String(Date.now()),
    name: editForm.name.trim(),
    kind,
    provider: kind === 'builtin' ? editForm.provider : (editForm.format === 'cc' ? 'custom-anthropic' : 'custom-openai'),
    format: kind === 'builtin' ? 'openai' : editForm.format,
    endpoint: kind === 'builtin' ? '' : editForm.endpoint.trim(),
    model: editForm.model.trim(),
  }
  if (kind === 'custom') {
    if (!profile.endpoint) return ElMessage.warning('请填写 API 地址')
    if (!profile.model) return ElMessage.warning('请填写模型名称')
  }
  if (editForm.apiKey.trim()) pendingKeys.set(profile.id, editForm.apiKey.trim())
  const idx = profiles.value.findIndex(p => p.id === profile.id)
  const view: AgentProfileView = {
    ...profile,
    apiKeyConfigured: editingKeyConfigured.value || !!editForm.apiKey.trim(),
    apiKeyRequired: profile.kind === 'builtin' ? builtinNeedsKey(profile.provider) : true,
  }
  if (idx >= 0) profiles.value[idx] = view
  else profiles.value.push(view)
  if (!active.value || active.value === editingId.value) active.value = profile.id
  editVisible.value = false
  await saveConfig()
}

const saveConfig = async () => {
  saving.value = true
  try {
    const payload: AgentProfileInput[] = profiles.value.map(p => ({
      id: p.id, name: p.name, kind: p.kind, provider: p.provider, format: p.format,
      endpoint: p.endpoint, model: p.model,
      apiKey: pendingKeys.get(p.id) || undefined,
    }))
    await agentApi.setConfig({ profiles: payload, active: active.value })
    pendingKeys.clear()
    await loadConfig()
    ElMessage.success('AI 配置已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : String(error))
  } finally { saving.value = false }
}
const activateProfile = async (id: string) => { active.value = id; await saveConfig() }
const removeProfile = async (id: string) => {
  if (profiles.value.length <= 1) return ElMessage.warning('至少需要保留一个 AI 配置')
  try {
    await ElMessageBox.confirm('删除该配置后其 API Key 将不再使用。确认删除？', '删除 AI 配置', { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' })
  } catch { return }
  profiles.value = profiles.value.filter(p => p.id !== id)
  if (active.value === id) active.value = profiles.value[0]?.id || ''
  pendingKeys.delete(id)
  await saveConfig()
}
const operationLabel = (operation: AgentChange['operation']) => ({ add: '新增记录', update: '修改记录', delete: '删除记录', set_setting: '系统设置', import: '导入数据' }[operation])
const riskyKeys = ['file_path', 'backup_path', 'path', 'endpoint', 'url']
const needsConfirm = (change: AgentChange) => {
  if (change.operation === 'delete' || change.operation === 'import') return true
  return Object.keys(change.fields).some(key => riskyKeys.includes(key))
}
const tableLabel = (change: AgentChange) => change.operation === 'set_setting' ? '系统设置' : change.operation === 'import' ? '导入' : change.table
const themeLabel = (mode: string) => ({ light: '浅色', dark: '深色', system: '跟随系统' }[mode] ?? mode)
const { setTheme } = useTheme()
const executeChange = async (change: AgentChange): Promise<string> => {
  if (change.operation === 'set_setting') {
    const theme = change.fields.theme
    if (theme === 'light' || theme === 'dark' || theme === 'system') {
      setTheme(theme)
      return `已切换主题为「${themeLabel(theme)}」`
    }
    const result = await agentApi.applyChange(change)
    if (result && result.success === false) throw new Error('配置变更执行失败')
    const applied = Array.isArray(result?.result) ? (result.result as string[]).join('、') : ''
    return applied ? `系统配置已更新：${applied}` : '系统配置已更新'
  }
  if (change.operation === 'import') {
    const filePath = change.fields.file_path
    return `导入操作需在界面手动完成（AI 无文件系统访问权限）。文件路径：${String(filePath ?? '未提供')}`
  }
  const result = await agentApi.applyChange(change)
  if (result && result.success === false) throw new Error('变更执行失败')
  const fields = change.fields as Record<string, unknown>
  const idLabel = change.id ? ` ${change.id}` : ''
  if (Object.keys(fields).length === 0) {
    return `已${operationLabel(change.operation)}${idLabel}（⚠️ 未指定要修改的字段）`
  }
  const fieldDescs = Object.entries(fields).map(([key, value]) => `${key} = ${String(value)}`).join('、')
  return `已${operationLabel(change.operation)}${idLabel}：${fieldDescs}`
}
// 批量执行结果 → 中文摘要（与 executeChange 的单条摘要格式一致）
const summaryOfApplied = (change: AgentChange, _applied: { operation: string; table: string }): string => {
  if (change.operation === 'set_setting') {
    const fields = change.fields as Record<string, unknown>
    const applied = Object.entries(fields).map(([key, value]) => `${key} = ${String(value)}`).join('、')
    return `系统配置已更新：${applied}`
  }
  const fields = change.fields as Record<string, unknown>
  const idLabel = change.id ? ` ${change.id}` : ''
  const fieldDescs = Object.entries(fields).map(([key, value]) => `${key} = ${String(value)}`).join('、')
  return `已${operationLabel(change.operation)}${idLabel}：${fieldDescs}`
}
const scrollBottom = async () => { await nextTick(); if (messagesRef.value) messagesRef.value.scrollTop = messagesRef.value.scrollHeight }
const askSuggestion = (text: string) => { question.value = text; send() }
// 重新生成：删除该条回复及其后的消息（含对应提问），按原请求类型重新发起。
// Excel 附件必须重新走 analyzeExcel（读取文件→JSON→专用命令），不能退化成只发路径的普通聊天。
const retryMessage = async (message: Message) => {
  if (loading.value) return
  const idx = messages.value.findIndex(m => m.id === message.id)
  if (idx < 0) return
  let userIdx = -1
  let userMessage: Message | null = null
  for (let i = idx - 1; i >= 0; i--) {
    if (messages.value[i].role === 'user') { userIdx = i; userMessage = messages.value[i]; break }
  }
  if (!userMessage) return
  const q = userMessage.content
  const attachment = userMessage.attachment
  messages.value = messages.value.slice(0, userIdx)
  if (attachment?.kind === 'excel') {
    await analyzeExcel(attachment.path)
    return
  }
  // 兼容旧版已保存的 Excel 消息（当时只有绝对路径，没有 attachment metadata）
  const legacyMatch = q.match(/^📎\s*分析 Excel 文件：([A-Za-z]:[\\/].+\.(?:xlsx|xls))$/i)
  if (legacyMatch?.[1]) {
    await analyzeExcel(legacyMatch[1])
    return
  }
  question.value = q
  await send()
}
// 对话历史传给后端前做字符预算裁剪：总字符超过 8000 时从最旧开始丢弃（至少保留最后一轮），
// 避免小上下文模型被历史撑爆。
const buildHistory = (): { role: 'user' | 'assistant'; content: string }[] => {
  const items = messages.value
    .filter((m) => m.role === 'user' || m.role === 'assistant')
    .map((m) => ({ role: m.role, content: m.content }))
  const MAX_CHARS = 8000
  let total = items.reduce((sum, m) => sum + m.content.length, 0)
  let start = 0
  while (start < items.length - 1 && total > MAX_CHARS) {
    total -= items[start].content.length
    start += 1
  }
  return items.slice(start)
}
// 公共发送骨架：push user 消息 → 流式 → 处理结果（changes 自动执行/待确认）
const sendWith = async (
  apiCall: () => Promise<AgentChatResult>,
  userContent: string,
  attachment?: Message['attachment'],
) => {
  if (loading.value) return
  messages.value.push({ id: Date.now(), role: 'user', content: userContent, attachment })
  loading.value = true
  streamReasoning.value = ''
  streamText.value = ''
  await scrollBottom()
  try {
    const result = await apiCall()
    const incoming = result.changes ?? null
    const msg: Message = { id: Date.now() + 1, role: 'assistant', content: result.answer, changes: null, reasoning: result.reasoning ?? null }
    messages.value.push(msg)
    const last = messages.value[messages.value.length - 1]
    if (incoming && incoming.length > 0) {
      await scrollBottom()
      const pending: AgentChange[] = []
      const summaries: string[] = []
      // 主题切换只能由界面执行，单独走单条逻辑；其余不需要确认的变更走后端批量接口
      // （一次备份 + 一次清理，避免大量变更逐条备份刷掉旧备份）。
      const auto = incoming.filter(c => !needsConfirm(c) && !(c.operation === 'set_setting' && typeof c.fields.theme === 'string'))
      const manual = incoming.filter(c => needsConfirm(c) || (c.operation === 'set_setting' && typeof c.fields.theme === 'string'))
      if (auto.length > 0) {
        try {
          const result = await agentApi.applyChanges(auto)
          // 后端在每条成功结果中携带 changeIndex，与 auto 精确对应；
          // 兼容旧后端（无 changeIndex）时按顺序回退。
          for (const applied of result.applied ?? []) {
            const changeIndex = typeof (applied as any)?.changeIndex === 'number' ? (applied as any).changeIndex : summaries.length
            const change = auto[changeIndex]
            if (change) summaries.push(summaryOfApplied(change, applied))
          }
          for (const failed of result.failed ?? []) {
            last.content += `\n\n自动执行失败：${failed.error}。如需执行请点击下方确认。`
            if (failed.change) pending.push(failed.change)
          }
        } catch (error) {
          last.content += `\n\n批量自动执行失败：${error instanceof Error ? error.message : String(error)}。如需执行请点击下方确认。`
          pending.push(...auto)
        }
      }
      for (const change of manual) {
        if (needsConfirm(change)) {
          pending.push(change)
        } else {
          try {
            summaries.push(await executeChange(change))
          } catch (error) {
            last.content += `\n\n自动执行失败：${error instanceof Error ? error.message : String(error)}。如需执行请点击下方确认。`
            pending.push(change)
          }
        }
      }
      if (summaries.length > 0) {
        last.content += `\n\n已批量自动执行 ${summaries.length} 条：\n${summaries.join('\n')}`
      }
      last.changes = pending.length > 0 ? pending : null
    } else {
      // 模型可能只在回答文字中声称“已修改/已更新”，但未提交任何可执行变更。
      // 明确提示用户：系统未修改任何数据，避免“假设置”误导。
      if (/已(?:修改|更新|执行|删除|新增|添加|设置)|已改为|已改成为/.test(result.answer)) {
        last.content += '\n\n⚠️ 本次回答没有附带任何可执行变更，系统未修改数据。若你认为修改未生效，请重新明确指示。'
      }
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    messages.value.push({ id: Date.now() + 1, role: 'assistant', content: `请求失败：${message}\n\n可以稍后重试，或检查「API 配置」中的服务商、模型名称与 API Key。` })
  } finally { loading.value = false; await scrollBottom() }
}
const send = async () => {
  const text = question.value.trim()
  if (!text || loading.value) return
  question.value = ''
  // 把当前对话历史（裁剪后）传给后端，让模型记住上下文
  await sendWith(() => agentApi.chat(text, buildHistory(), pageContext.value), text)
}
// 上传 Excel：仅把 Excel 完整转换为 JSON 后交给 AI（不预处理差异、不导入数据库）
const analyzeExcel = async (path: string) => {
  const name = path.split(/[\\/]/).pop() || 'Excel 文件'
  await sendWith(
    () => agentApi.analyzeExcel(path, buildHistory(), pageContext.value),
    `📎 分析 Excel 文件：${name}`,
    { kind: 'excel', path, name },
  )
}
const applyChange = async (message: Message, change: AgentChange, index: number) => {
  if (!change) return
  if (needsConfirm(change)) {
    const riskText = change.operation === 'import'
      ? '确认要导入该文件吗？导入将按表替换现有数据，请确认文件与目标表正确。'
      : '确认将此变更写入系统？执行前会自动备份当前数据。'
    try {
      await ElMessageBox.confirm(riskText, '确认 AI 变更', { type: 'warning', confirmButtonText: '确认执行', cancelButtonText: '取消' })
    } catch { return }
  }
  applyingId.value = `${message.id}-${index}`
  try {
    const summary = await executeChange(change)
    ElMessage.success(summary)
    message.changes?.splice(index, 1)
    if (!message.changes || message.changes.length === 0) message.changes = null
    message.content += `\n\n${summary}。`
  } catch (error) { ElMessage.error(error instanceof Error ? error.message : String(error)) }
  finally { applyingId.value = null }
}
const discardChange = (message: Message, _change: AgentChange, index: number) => {
  message.changes?.splice(index, 1)
  if (!message.changes || message.changes.length === 0) message.changes = null
  message.content += '\n\n已取消这项变更。'
}
const clearChat = async () => {
  try {
    await ElMessageBox.confirm('将清空当前对话的消息（不影响已保存的业务数据）。', '清空对话', { type: 'warning', confirmButtonText: '清空', cancelButtonText: '取消' })
  } catch { return }
  messages.value = []
  await scrollBottom()
}
const loadConfig = async () => {
  const result = await agentApi.getConfig()
  profiles.value = result.profiles ?? []
  active.value = result.active || (profiles.value[0]?.id ?? '')
  if (result.builtins && result.builtins.length > 0) builtins.value = result.builtins
  const backendAll = normalizeZenModels(result.zenModels ?? result.ccModels)
  const backendFree = normalizeZenModels(result.zenFreeModels)
  // 新后端返回完整对象列表时优先使用；旧后端只有少量字符串候选时，用前端完整预设兜底。
  zenModels.value = backendAll.length >= ZEN_MODEL_PRESETS.length / 2 ? backendAll : [...ZEN_MODEL_PRESETS]
  zenFreeModels.value = backendFree.length ? backendFree : [...ZEN_FREE_MODEL_PRESETS]
}
// ===== 上传 Excel 直接交给 AI 分析（不做差异预处理） =====
const uploadExcel = async () => {
  try {
    const filePath = await open({ title: '选择要分析的 Excel 文件', filters: [{ name: 'Excel', extensions: ['xlsx', 'xls'] }], multiple: false })
    if (!filePath) return
    await analyzeExcel(String(filePath))
  } catch { /* 取消选择 */ }
}
onMounted(() => {
  loadSessions()
  loadConfig()
  buildPageContext()
  setTimeout(() => scrollBottom(), 50)
  // 监听后端流式输出："r:" 前缀为思考过程增量，其余为回答正文增量
  listen<string>('agent-stream', (event) => {
    const chunk = event.payload ?? ''
    if (chunk.startsWith('r:')) {
      streamReasoning.value += chunk.slice(2)
    } else if (chunk) {
      streamText.value += chunk
    }
    scrollBottom()
  }).then((unlisten) => { unlistenStream = unlisten }).catch(() => { /* 非 Tauri 环境忽略 */ })
})
onUnmounted(() => { if (unlistenStream) { unlistenStream(); unlistenStream = null } })
</script>

<style scoped>
.agent-page { height:100%; overflow:hidden; padding:4px 4px 24px; box-sizing:border-box; display:flex; flex-direction:column; }
.agent-header { display:flex; justify-content:space-between; align-items:flex-start; gap:20px; margin-bottom:16px; flex-shrink:0; }
.eyebrow { color:var(--primary); font-size:11px; font-weight:700; letter-spacing:2px; margin-bottom:6px; }
h1 { display:flex; align-items:center; gap:10px; margin:0; color:var(--text-primary); font-size:26px; }
.agent-header p { margin:8px 0 0; color:var(--text-secondary); }
.agent-header-actions { display:flex; gap:10px; }
.agent-grid { display:grid; grid-template-columns:minmax(0, 1fr); gap:18px; flex:1; min-height:0; }
.agent-grid.has-sidebar { grid-template-columns:220px minmax(0, 1fr); }
.session-sidebar { height:100%; min-height:0; display:flex; flex-direction:column; }
.session-sidebar :deep(.el-card__body) { display:flex; flex-direction:column; min-height:0; flex:1; padding:14px; }
.session-toolbar { margin-bottom:10px; }
.new-session-btn { width:100%; }
.session-list { flex:1; min-height:0; overflow-y:auto; display:flex; flex-direction:column; gap:4px; }
.session-item { display:flex; align-items:center; gap:4px; padding:8px 10px; border-radius:8px; cursor:pointer; color:var(--text-primary); }
.session-item:hover { background:var(--surface-muted); }
.session-item.is-active { background:var(--primary-faint, var(--surface-muted)); color:var(--primary); font-weight:600; }
.session-title { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:13px; }
.session-del { flex-shrink:0; }
.session-empty { color:var(--text-muted); font-size:12px; text-align:center; padding:14px 0; }
.session-hint { margin-top:8px; color:var(--text-muted); font-size:11px; text-align:center; }
.compare-dialog :deep(.el-dialog__body) { max-height:60vh; overflow-y:auto; }
.compare-loading { color:var(--text-secondary); padding:20px 0; text-align:center; }
.compare-empty { color:var(--text-muted); padding:20px 0; text-align:center; }
.compare-table { border:1px solid var(--border); border-radius:10px; padding:12px 14px; margin-bottom:12px; }
.compare-table-head { display:flex; align-items:center; gap:8px; flex-wrap:wrap; margin-bottom:8px; }
.compare-table-name { font-weight:650; color:var(--text-primary); margin-right:4px; }
.compare-skip { color:var(--text-muted); font-size:12px; }
.compare-same { color:var(--success); font-size:12px; }
.compare-detail { font-size:12px; color:var(--text-secondary); }
.compare-section-title { font-weight:600; color:var(--text-primary); margin:10px 0 6px; }
.compare-row { padding:3px 0; word-break:break-all; }
.compare-mod-key { font-weight:600; color:var(--text-primary); margin-bottom:2px; }
.compare-mod-field span { color:var(--text-muted); }
.compare-mod-field b.old { color:var(--danger); font-weight:500; }
.compare-mod-field b.new { color:var(--success); font-weight:500; }
.chat-card { height:100%; min-height:0; }
.chat-card { display:flex; flex-direction:column; }
.chat-card :deep(.el-card__body) { display:flex; flex-direction:column; min-height:0; flex:1; padding:0; }
.card-header { display:flex; justify-content:space-between; align-items:center; font-weight:650; color:var(--text-primary); }
.card-header span { display:flex; align-items:center; gap:8px; }
.card-header-actions { display:flex; align-items:center; gap:10px; }
.messages { flex:1; min-height:0; overflow-y:auto; padding:24px; }
.empty-chat { display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:330px; gap:10px; color:var(--text-muted); text-align:center; }
.empty-chat strong { color:var(--text-primary); font-size:17px; }
.suggestions { display:flex; gap:8px; flex-wrap:wrap; justify-content:center; margin-top:8px; }
.message-row { display:flex; gap:10px; margin-bottom:18px; align-items:flex-start; }
.message-row.user { flex-direction:row-reverse; }
.message-avatar { flex:0 0 30px; width:30px; height:30px; display:grid; place-items:center; border-radius:9px; color:#fff; background:var(--primary); }
.message-row.user .message-avatar { background:#64748b; }
.message-bubble { max-width:min(78%, 720px); padding:12px 15px; border-radius:12px; background:var(--surface-muted); color:var(--text-primary); line-height:1.65; white-space:pre-wrap; }
.message-row.user .message-bubble { background:var(--primary); color:#fff; }
.loading-bubble { color:var(--text-secondary); }
.reasoning-block { margin-bottom:10px; border:1px solid var(--border); border-radius:8px; background:var(--surface-muted); padding:8px 12px; }
.reasoning-live { margin-bottom:10px; border:1px solid var(--border); border-radius:8px; background:var(--surface-muted); padding:8px 12px; }
.reasoning-live-title { display:flex; align-items:center; gap:6px; font-size:12px; color:var(--text-secondary); margin-bottom:6px; }
.reasoning-live-title .el-icon { color:var(--primary); }
.stream-text { white-space:pre-wrap; word-break:break-word; }
.stream-text::after { content: '▍'; color:var(--primary); animation:blink 1s step-start infinite; }
@keyframes blink { 50% { opacity:0; } }
.message-actions { margin-top:8px; display:flex; justify-content:flex-end; }
.message-actions :deep(.el-button) { font-size:12px; }
.reasoning-block summary { cursor:pointer; display:flex; align-items:center; gap:6px; font-size:12px; color:var(--text-secondary); user-select:none; }
.reasoning-block summary .el-icon { color:var(--primary); }
.reasoning-text { margin-top:8px; font-size:12px; line-height:1.7; color:var(--text-secondary); white-space:pre-wrap; max-height:200px; overflow:auto; }
.composer { padding:14px 18px 18px; border-top:1px solid var(--border); background:var(--surface-muted); }
.composer-footer { display:flex; justify-content:space-between; align-items:center; gap:12px; margin-top:10px; color:var(--text-muted); font-size:12px; }
.change-card { margin-top:12px; padding:12px; border:1px solid var(--border-strong); border-radius:10px; background:var(--card-bg); color:var(--text-primary); }
.change-title { display:flex; align-items:center; gap:7px; font-weight:650; margin-bottom:10px; }
.field-change { display:flex; justify-content:space-between; gap:18px; }
.change-actions { display:flex; justify-content:flex-end; gap:8px; margin-top:10px; }
.provider-select { width:100%; }
.provider-alert { margin:0 0 18px 100px; width:calc(100% - 100px); box-sizing:border-box; }
.free-key-hint { color:var(--text-muted); font-size:12px; }
.model-id { float:right; margin-left:16px; color:var(--text-muted); font-size:12px; }
.profile-list { display:flex; flex-direction:column; gap:10px; max-height:320px; overflow-y:auto; padding-right:4px; }
.profile-item { display:flex; align-items:center; justify-content:space-between; gap:12px; padding:12px 14px; border:1px solid var(--border); border-radius:10px; background:var(--surface-muted); }
.profile-item.is-active { border-color:var(--primary); background:var(--primary-faint, transparent); }
.profile-main { display:flex; align-items:center; gap:8px; }
.profile-name { font-weight:650; color:var(--text-primary); }
.profile-meta { flex:1; min-width:0; color:var(--text-secondary); font-size:12px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.profile-actions { display:flex; gap:6px; flex-shrink:0; }
.profile-empty { color:var(--text-muted); font-size:13px; text-align:center; padding:16px 0; }
.add-profile-btn { align-self:flex-start; }
</style>
