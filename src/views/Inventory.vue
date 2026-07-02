<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon><DataAnalysis /></el-icon>
          <span>库存汇总</span>
        </div>
      </template>

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
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
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
.inventory-tabs {
  margin-top: 8px;
}
</style>
