<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><DataAnalysis /></el-icon>
          <span>库存汇总</span>
        </div>
      </template>

      <div class="inventory-content">
        <div class="inventory-intro">
          <div>
            <h2>库存状态总览</h2>
            <p>按物料类别查看当前库存、安全库存与补货状态。</p>
          </div>
          <span class="inventory-count">{{ activeStockCount }} 项</span>
        </div>

        <el-tabs v-model="activeTab" class="inventory-tabs">
          <el-tab-pane label="冲头库存" name="punch">
            <StockTable :data="punchStock" name-label="冲头名称" :loading="loading" />
          </el-tab-pane>
          <el-tab-pane label="牙板库存" name="die">
            <StockTable :data="dieStock" name-label="牙板名称" :loading="loading" />
          </el-tab-pane>
          <el-tab-pane label="皮带库存" name="belt">
            <StockTable :data="beltStock" name-label="皮带名称" :loading="loading" />
          </el-tab-pane>
          <el-tab-pane label="主模具库存" name="mainMold">
            <StockTable :data="mainMoldStock" name-label="主模具名称" :loading="loading" />
          </el-tab-pane>
          <el-tab-pane label="剪刀库存" name="scissor">
            <StockTable :data="scissorStock" name-label="剪刀名称" :loading="loading" />
          </el-tab-pane>
          <el-tab-pane label="上冲库存" name="upperPunch">
            <StockTable :data="upperPunchStock" name-label="上冲名称" :loading="loading" />
          </el-tab-pane>
        </el-tabs>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { stockCalcApi } from '../api'
import StockTable from '../components/StockTable.vue'

const activeTab = ref('punch')
const punchStock = ref<any[]>([])
const dieStock = ref<any[]>([])
const beltStock = ref<any[]>([])
const mainMoldStock = ref<any[]>([])
const scissorStock = ref<any[]>([])
const upperPunchStock = ref<any[]>([])
const loading = ref(true)

const activeStockCount = computed(() => {
  const stockByTab: Record<string, any[]> = {
    punch: punchStock.value,
    die: dieStock.value,
    belt: beltStock.value,
    mainMold: mainMoldStock.value,
    scissor: scissorStock.value,
    upperPunch: upperPunchStock.value,
  }

  return stockByTab[activeTab.value]?.length ?? 0
})

onMounted(async () => {
  loading.value = true
  try {
    const result = await stockCalcApi.calculateAll()
    punchStock.value = result.punch || []
    dieStock.value = result.die || []
    beltStock.value = result.belt || []
    mainMoldStock.value = result.mainMold || []
    scissorStock.value = result.scissor || []
    upperPunchStock.value = result.upperPunch || []
  } catch (error) {
    ElMessage.error('加载库存数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.inventory-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 4px 2px 2px;
}

.inventory-intro {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 8px 4px 18px;
}

.inventory-intro h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.inventory-intro p {
  margin: 5px 0 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.inventory-count {
  flex: none;
  padding: 7px 12px;
  color: var(--el-color-primary);
  font-size: 13px;
  font-weight: 600;
  background: var(--el-color-primary-light-9);
  border: 1px solid var(--el-color-primary-light-7);
  border-radius: 999px;
}

.inventory-tabs {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.inventory-tabs :deep(.el-tabs__header) {
  margin-bottom: 14px;
}

.inventory-tabs :deep(.el-tabs__content) {
  flex: 1;
  min-height: 0;
}

.inventory-tabs :deep(.el-tab-pane) {
  height: 100%;
  min-height: 0;
}

@media (max-width: 720px) {
  .inventory-intro {
    align-items: flex-start;
    padding-bottom: 14px;
  }

  .inventory-intro p {
    max-width: 32ch;
  }
}
</style>
