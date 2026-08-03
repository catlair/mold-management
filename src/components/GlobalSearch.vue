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
      />
      <div class="search-results" v-loading="loading">
        <div v-if="!keyword && !loading" class="search-empty">
          输入关键词开始搜索
        </div>
        <div v-else-if="groups.length === 0 && !loading && keyword" class="search-empty">
          未找到匹配结果
        </div>
        <template v-for="group in groups" :key="group.label">
          <div class="result-group">
            <div class="group-label">{{ group.label }}</div>
            <div
              v-for="item in group.items"
              :key="item.id"
              class="result-item"
              role="button"
              tabindex="0"
              @click="goTo(item)"
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
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
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

defineProps<{ collapsed?: boolean }>()

let searchTimer: any = null

function openSearch() {
  visible.value = true
  nextTick(() => {
    setTimeout(() => {
      searchInputRef.value?.focus()
    }, 100)
  })
}

function onDialogClose() {
  keyword.value = ''
}

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

function onSearch() {
  clearTimeout(searchTimer)
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
    const { values, failures } = await settleNamedRequests([
      { label: '螺丝规格', request: screwSpecApi.getAll() },
      { label: '冲头', request: punchApi.getAll() },
      { label: '牙板', request: dieApi.getAll() },
      { label: '皮带', request: beltApi.getAll() },
      { label: '主模具', request: mainMoldApi.getAll() },
      { label: '剪刀', request: scissorApi.getAll() },
      { label: '上冲', request: upperPunchApi.getAll() },
    ])
    const [screws, punches, dies, belts, molds, scissors, upperPunches] = values.map(
      value => Array.isArray(value) ? value : [],
    )

    const result: SearchGroup[] = []

    // 螺丝规格
    const screwMatches = screws.filter((s: any) => matchFields(kw, [s.name, s.headType, s.threadType, s.punch, s.die, s.wireMaterial, s.externalId, s.customer, s.remark]))
    if (screwMatches.length) {
      result.push({
        label: `螺丝规格 (${screwMatches.length})`,
        color: '#409eff',
        icon: Document,
        items: screwMatches.map((s: any) => ({
          id: s.id, name: s.name,
          desc: [s.wireMaterial, s.externalId, s.customer, s.headType, s.punch && `冲头:${s.punch}`, s.die && `牙板:${s.die}`].filter(Boolean).join(' · '),
          route: '/screw-spec'
        }))
      })
    }

    // 冲头
    const punchMatches = punches.filter((p: any) => matchFields(kw, [p.name, p.spec, p.material]))
    if (punchMatches.length) {
      result.push({
        label: `冲头 (${punchMatches.length})`,
        color: '#e6a23c',
        icon: SetUp,
        items: punchMatches.map((p: any) => ({
          id: p.id, name: p.name,
          desc: [p.spec, p.material].filter(Boolean).join(' · '),
          route: '/punch'
        }))
      })
    }

    // 牙板
    const dieMatches = dies.filter((d: any) => matchFields(kw, [d.name, d.machineType, d.wireDiameter]))
    if (dieMatches.length) {
      result.push({
        label: `牙板 (${dieMatches.length})`,
        color: '#67c23a',
        icon: Grid,
        items: dieMatches.map((d: any) => ({
          id: d.id, name: d.name,
          desc: [d.machineType, d.wireDiameter && `线径${d.wireDiameter}`].filter(Boolean).join(' · '),
          route: '/die'
        }))
      })
    }

    // 皮带
    const beltMatches = belts.filter((b: any) => matchFields(kw, [b.name, b.machine]))
    if (beltMatches.length) {
      result.push({
        label: `皮带 (${beltMatches.length})`,
        color: '#909399',
        icon: Connection,
        items: beltMatches.map((b: any) => ({
          id: b.id, name: b.name,
          desc: b.machine || '',
          route: '/belt'
        }))
      })
    }

    // 主模具
    const moldMatches = molds.filter((m: any) => matchFields(kw, [m.name, m.holeDiameter, m.wireMaterial]))
    if (moldMatches.length) {
      result.push({
        label: `主模具 (${moldMatches.length})`,
        color: '#f56c6c',
        icon: Box,
        items: moldMatches.map((m: any) => ({
          id: m.id, name: m.name,
          desc: [m.holeDiameter, m.wireMaterial].filter(Boolean).join(' · '),
          route: '/main-mold'
        }))
      })
    }

    // 剪刀
    const scissorMatches = scissors.filter((s: any) => matchFields(kw, [s.name, s.diameter, s.wireMaterial]))
    if (scissorMatches.length) {
      result.push({
        label: `剪刀 (${scissorMatches.length})`,
        color: '#909399',
        icon: Scissor,
        items: scissorMatches.map((s: any) => ({
          id: s.id, name: s.name,
          desc: [s.diameter, s.wireMaterial].filter(Boolean).join(' · '),
          route: '/scissor'
        }))
      })
    }

    // 上冲
    const upperMatches = upperPunches.filter((u: any) => matchFields(kw, [u.name, u.diameter, u.wireMaterial]))
    if (upperMatches.length) {
      result.push({
        label: `上冲 (${upperMatches.length})`,
        color: '#e6a23c',
        icon: Top,
        items: upperMatches.map((u: any) => ({
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
  const nkw = normalize(kw)
  return fields.some(f => {
    if (!f) return false
    return normalize(String(f)).includes(nkw)
  })
}

function goTo(item: SearchResult) {
  visible.value = false
  keyword.value = ''
  router.push({ path: item.route, query: { highlight: item.id } })
}

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    openSearch()
  }
}

onMounted(() => { document.addEventListener('keydown', onKeydown) })
onUnmounted(() => { document.removeEventListener('keydown', onKeydown) })
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
.result-item:hover {
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
