<template>
  <div class="global-search">
    <el-input
      v-if="!collapsed"
      v-model="keyword"
      placeholder="搜索... (Ctrl+K)"
      prefix-icon="Search"
      clearable
      @click="openSearch"
      readonly
      tabindex="-1"
      class="search-input"
    />
    <button
      v-else
      type="button"
      class="search-icon-btn"
      title="搜索 (Ctrl+K)"
      aria-label="搜索"
      @click="openSearch"
    >
      <el-icon><Search /></el-icon>
    </button>
    <el-dialog
      v-model="visible"
      width="600px"
      :show-close="false"
      class="search-dialog"
      @close="onDialogClose"
    >
      <el-input
        ref="searchInputRef"
        v-model="keyword"
        placeholder="输入关键词搜索螺丝规格、冲头、牙板..."
        prefix-icon="Search"
        clearable
        size="large"
        @input="onSearch"
        @keydown="onInputKeydown"
      />
      <div class="search-results" v-loading="loading">
        <!-- 搜索历史（输入为空时显示） -->
        <template v-if="!keyword && !loading">
          <div v-if="searchHistory.length" class="search-history">
            <div class="history-header">
              <span class="history-title">最近搜索</span>
              <button class="history-clear" @click="clearHistory">清除</button>
            </div>
            <div class="history-tags">
              <span
                v-for="h in searchHistory"
                :key="h"
                class="history-tag"
                @click="keyword = h; onSearch()"
              >{{ h }}</span>
            </div>
          </div>
          <div v-else class="search-empty">
            输入关键词开始搜索
          </div>
        </template>
        <!-- 搜索结果 -->
        <template v-else-if="groups.length === 0 && !loading && keyword">
          <div class="search-empty">未找到匹配结果</div>
        </template>
        <template v-else>
          <template v-for="(group, gi) in groups" :key="group.label">
            <div class="result-group">
              <div class="group-label">{{ group.label }}</div>
              <div
                v-for="(item, ii) in group.items"
                :key="item.id"
                class="result-item"
                :class="{ 'result-item--active': flatIndex(gi, ii) === activeIndex }"
                role="button"
                tabindex="0"
                @click="goTo(item)"
                @mouseenter="activeIndex = flatIndex(gi, ii)"
                @keydown.enter="goTo(item)"
                @keydown.space.prevent="goTo(item)"
              >
                <el-icon class="result-icon" :style="{ color: group.color }"><component :is="group.icon" /></el-icon>
                <div class="result-info">
                  <div class="result-name">{{ item.name }}</div>
                  <div class="result-desc">{{ item.desc }}</div>
                </div>
                <el-icon class="result-arrow"><ArrowRight /></el-icon>
              </div>
            </div>
          </template>
        </template>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Document, SetUp, Grid, Connection, Box, Scissor, Top } from '@element-plus/icons-vue'
import {
  screwSpecApi, punchApi, dieApi, beltApi, mainMoldApi, scissorApi, upperPunchApi
} from '../api'
import { settleNamedRequests, showBatchErrors } from '../utils/errorFeedback'

const router = useRouter()
const visible = ref(false)
const keyword = ref('')
const loading = ref(false)
const searchInputRef = ref<any>(null)
const groups = ref<any[]>([])
const activeIndex = ref(-1)

defineProps<{ collapsed?: boolean }>()

let searchTimer: any = null

// ─── 数据缓存 ────────────────────────────────────────
// 模块级缓存：避免每次搜索都全量加载 7 张表
interface DataCache {
  data: any[][]  // [screws, punches, dies, belts, molds, scissors, upperPunches]
  timestamp: number
}
let dataCache: DataCache | null = null
const CACHE_TTL = 60_000 // 60 秒缓存

async function loadAllData(): Promise<{ data: any[][]; failures: any[] }> {
  if (dataCache && Date.now() - dataCache.timestamp < CACHE_TTL) {
    return { data: dataCache.data, failures: [] }
  }
  const { values, failures } = await settleNamedRequests([
    { label: '螺丝规格', request: screwSpecApi.getAll() },
    { label: '冲头', request: punchApi.getAll() },
    { label: '牙板', request: dieApi.getAll() },
    { label: '皮带', request: beltApi.getAll() },
    { label: '主模具', request: mainMoldApi.getAll() },
    { label: '剪刀', request: scissorApi.getAll() },
    { label: '上冲', request: upperPunchApi.getAll() },
  ])
  const data = values.map(v => Array.isArray(v) ? v : [])
  dataCache = { data, timestamp: Date.now() }
  return { data, failures }
}

/** 对外暴露缓存失效方法（数据增删改后可调用） */
function invalidateSearchCache() {
  dataCache = null
}

// ─── 搜索历史 ────────────────────────────────────────
const HISTORY_KEY = 'mold-management.search-history'
const HISTORY_MAX = 10
const searchHistory = ref<string[]>([])

function loadHistory() {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    searchHistory.value = raw ? JSON.parse(raw) : []
  } catch {
    searchHistory.value = []
  }
}

function saveHistory(kw: string) {
  const trimmed = kw.trim()
  if (!trimmed) return
  // 去重：移除已存在的相同关键词，再放到最前
  const filtered = searchHistory.value.filter(h => h !== trimmed)
  filtered.unshift(trimmed)
  searchHistory.value = filtered.slice(0, HISTORY_MAX)
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(searchHistory.value))
  } catch { /* ignore */ }
}

function clearHistory() {
  searchHistory.value = []
  try { localStorage.removeItem(HISTORY_KEY) } catch { /* ignore */ }
}

// ─── 搜索逻辑 ────────────────────────────────────────
interface SearchResult {
  id: string
  name: string
  desc: string
  route: string
}

interface SearchGroup {
  label: string
  color: string
  icon: any
  items: SearchResult[]
}

const MAX_PER_GROUP = 10

function onSearch() {
  clearTimeout(searchTimer)
  activeIndex.value = -1
  if (!keyword.value.trim()) {
    groups.value = []
    return
  }
  searchTimer = setTimeout(doSearch, 300)
}

async function doSearch() {
  const kw = normalize(keyword.value.trim())
  if (!kw) return
  loading.value = true
  try {
    const { data, failures } = await loadAllData()
    const [screws, punches, dies, belts, molds, scissors, upperPunches] = data

    const result: SearchGroup[] = []

    // 螺丝规格 —— 搜索全部文本字段
    const screwMatches = screws.filter((s: any) => matchFields(kw, [
      s.name, s.headType, s.threadType, s.punch, s.die,
      s.wireMaterial, s.externalId, s.customer, s.remark,
      s.headSize, s.headHeight, s.length, s.threadDiameter,
      s.shankLength, s.plating,
    ]))
    if (screwMatches.length) {
      result.push({
        label: `螺丝规格 (${screwMatches.length})`,
        color: '#409eff',
        icon: Document,
        items: screwMatches.slice(0, MAX_PER_GROUP).map((s: any) => ({
          id: s.id, name: s.name,
          desc: [s.wireMaterial, s.externalId, s.customer, s.headType, s.punch && `冲头:${s.punch}`, s.die && `牙板:${s.die}`].filter(Boolean).join(' · '),
          route: '/screw-spec'
        }))
      })
    }

    // 冲头
    const punchMatches = punches.filter((p: any) => matchFields(kw, [p.name, p.spec, p.material, p.remark]))
    if (punchMatches.length) {
      result.push({
        label: `冲头 (${punchMatches.length})`,
        color: '#e6a23c',
        icon: SetUp,
        items: punchMatches.slice(0, MAX_PER_GROUP).map((p: any) => ({
          id: p.id, name: p.name,
          desc: [p.spec, p.material].filter(Boolean).join(' · '),
          route: '/punch'
        }))
      })
    }

    // 牙板
    const dieMatches = dies.filter((d: any) => matchFields(kw, [d.name, d.machineType, d.wireDiameter, d.remark]))
    if (dieMatches.length) {
      result.push({
        label: `牙板 (${dieMatches.length})`,
        color: '#67c23a',
        icon: Grid,
        items: dieMatches.slice(0, MAX_PER_GROUP).map((d: any) => ({
          id: d.id, name: d.name,
          desc: [d.machineType, d.wireDiameter && `线径${d.wireDiameter}`].filter(Boolean).join(' · '),
          route: '/die'
        }))
      })
    }

    // 皮带
    const beltMatches = belts.filter((b: any) => matchFields(kw, [b.name, b.machine, b.remark]))
    if (beltMatches.length) {
      result.push({
        label: `皮带 (${beltMatches.length})`,
        color: '#909399',
        icon: Connection,
        items: beltMatches.slice(0, MAX_PER_GROUP).map((b: any) => ({
          id: b.id, name: b.name,
          desc: b.machine || '',
          route: '/belt'
        }))
      })
    }

    // 主模具
    const moldMatches = molds.filter((m: any) => matchFields(kw, [m.name, m.holeDiameter, m.wireMaterial, m.remark]))
    if (moldMatches.length) {
      result.push({
        label: `主模具 (${moldMatches.length})`,
        color: '#f56c6c',
        icon: Box,
        items: moldMatches.slice(0, MAX_PER_GROUP).map((m: any) => ({
          id: m.id, name: m.name,
          desc: [m.holeDiameter, m.wireMaterial].filter(Boolean).join(' · '),
          route: '/main-mold'
        }))
      })
    }

    // 剪刀
    const scissorMatches = scissors.filter((s: any) => matchFields(kw, [s.name, s.diameter, s.wireMaterial, s.remark]))
    if (scissorMatches.length) {
      result.push({
        label: `剪刀 (${scissorMatches.length})`,
        color: '#909399',
        icon: Scissor,
        items: scissorMatches.slice(0, MAX_PER_GROUP).map((s: any) => ({
          id: s.id, name: s.name,
          desc: [s.diameter, s.wireMaterial].filter(Boolean).join(' · '),
          route: '/scissor'
        }))
      })
    }

    // 上冲
    const upperMatches = upperPunches.filter((u: any) => matchFields(kw, [u.name, u.diameter, u.wireMaterial, u.remark]))
    if (upperMatches.length) {
      result.push({
        label: `上冲 (${upperMatches.length})`,
        color: '#e6a23c',
        icon: Top,
        items: upperMatches.slice(0, MAX_PER_GROUP).map((u: any) => ({
          id: u.id, name: u.name,
          desc: [u.diameter, u.wireMaterial].filter(Boolean).join(' · '),
          route: '/upper-punch'
        }))
      })
    }

    groups.value = result
    showBatchErrors('全局搜索数据加载', failures)
  } finally {
    loading.value = false
  }
}

function normalize(s: string): string {
  return s.toLowerCase().replace(/\s*[xX×]\s*/g, 'x')
}

function matchFields(kw: string, fields: any[]): boolean {
  return fields.some(f => {
    if (!f) return false
    return normalize(String(f)).includes(kw)
  })
}

// ─── 键盘导航 ────────────────────────────────────────
/** 计算扁平化索引：将 (groupIndex, itemIndex) 映射为全局序号 */
function flatIndex(gi: number | string, ii: number | string): number {
  const g = Number(gi)
  const i = Number(ii)
  let count = 0
  for (let k = 0; k < g; k++) count += groups.value[k]?.items.length || 0
  return count + i
}

/** 总结果数 */
const totalResults = computed(() => groups.value.reduce((sum, g) => sum + g.items.length, 0))

function onInputKeydown(e: KeyboardEvent) {
  if (totalResults.value === 0) return
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    activeIndex.value = Math.min(activeIndex.value + 1, totalResults.value - 1)
    scrollActiveIntoView()
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    activeIndex.value = Math.max(activeIndex.value - 1, 0)
    scrollActiveIntoView()
  } else if (e.key === 'Enter' && activeIndex.value >= 0) {
    e.preventDefault()
    const item = getActiveItem()
    if (item) goTo(item)
  }
}

function getActiveItem(): SearchResult | null {
  let remaining = activeIndex.value
  for (const g of groups.value) {
    if (remaining < g.items.length) return g.items[remaining]
    remaining -= g.items.length
  }
  return null
}

function scrollActiveIntoView() {
  nextTick(() => {
    const el = document.querySelector('.result-item--active') as HTMLElement | null
    el?.scrollIntoView({ block: 'nearest' })
  })
}

// ─── 导航 ────────────────────────────────────────────
function goTo(item: SearchResult) {
  // 保存搜索历史
  if (keyword.value.trim()) saveHistory(keyword.value.trim())
  visible.value = false
  keyword.value = ''
  router.push({ path: item.route, query: { highlight: item.id } })
}

// ─── 对话框 & 快捷键 ──────────────────────────────────
function openSearch() {
  visible.value = true
  loadHistory()
  nextTick(() => {
    setTimeout(() => {
      searchInputRef.value?.focus()
    }, 100)
  })
}

function onDialogClose() {
  keyword.value = ''
  groups.value = []
  activeIndex.value = -1
}

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    openSearch()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
  loadHistory()
})
onUnmounted(() => { document.removeEventListener('keydown', onKeydown) })

// 导出缓存失效方法供外部使用
defineExpose({ invalidateSearchCache })
</script>

<style scoped>
.search-input {
  cursor: pointer;
}
.search-input :deep(.el-input__wrapper) {
  background: color-mix(in srgb, var(--card-bg) 72%, transparent);
  border-radius: 6px;
}
.search-icon-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--sidebar-text);
  background: transparent;
  cursor: pointer;
  transition: color 0.2s ease, background-color 0.2s ease, border-color 0.2s ease;
}
.search-icon-btn .el-icon {
  font-size: 18px;
}
.search-icon-btn:hover {
  color: var(--primary-light);
  background: var(--sidebar-hover);
}
.search-icon-btn:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}
.search-dialog :deep(.el-dialog__header) {
  display: none;
}
.search-dialog :deep(.el-dialog__body) {
  padding: 16px;
}
.search-results {
  margin-top: 12px;
  max-height: 400px;
  overflow-y: auto;
}
.search-empty {
  text-align: center;
  color: var(--text-muted);
  padding: 24px 0;
  font-size: 14px;
}
/* 搜索历史 */
.search-history {
  padding: 4px 0;
}
.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.history-title {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 600;
}
.history-clear {
  font-size: 12px;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: color 0.15s, background-color 0.15s;
}
.history-clear:hover {
  color: var(--primary-light);
  background: var(--surface-hover);
}
.history-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.history-tag {
  display: inline-block;
  padding: 4px 12px;
  font-size: 13px;
  border-radius: 14px;
  background: var(--surface-hover);
  color: var(--text-secondary);
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
  user-select: none;
}
.history-tag:hover {
  background: color-mix(in srgb, var(--primary) 12%, var(--surface-hover));
  color: var(--primary-light);
}
.result-group {
  margin-bottom: 12px;
}
.group-label {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 600;
  padding: 4px 0;
  border-bottom: 1px solid var(--border);
  margin-bottom: 4px;
}
.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.15s ease, transform 0.15s ease;
}
.result-item:hover,
.result-item--active {
  background: var(--surface-hover);
}
.result-item:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: -2px;
  background: var(--surface-hover);
}
.result-item:active {
  transform: scale(0.995);
}
.result-icon {
  font-size: 18px;
  flex-shrink: 0;
}
.result-info {
  flex: 1;
  min-width: 0;
}
.result-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}
.result-desc {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.result-arrow {
  color: var(--text-muted);
  font-size: 14px;
  flex-shrink: 0;
}
</style>
