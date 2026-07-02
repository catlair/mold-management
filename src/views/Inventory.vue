<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <span>库存汇总</span>
      </template>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="冲头库存" name="punch">
          <el-table :data="punchStock" border style="width: 100%" v-loading="loading">
            <el-table-column prop="name" label="冲头名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="牙板库存" name="die">
          <el-table :data="dieStock" border style="width: 100%">
            <el-table-column prop="name" label="牙板名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="皮带库存" name="belt">
          <el-table :data="beltStock" border style="width: 100%">
            <el-table-column prop="name" label="皮带名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="主模具库存" name="mainMold">
          <el-table :data="mainMoldStock" border style="width: 100%">
            <el-table-column prop="name" label="主模具名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="剪刀库存" name="scissor">
          <el-table :data="scissorStock" border style="width: 100%">
            <el-table-column prop="name" label="剪刀名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="上冲库存" name="upperPunch">
          <el-table :data="upperPunchStock" border style="width: 100%">
            <el-table-column prop="name" label="上冲名称" width="120" />
            <el-table-column prop="currentStock" label="当前库存" width="100" />
            <el-table-column prop="safetyStock" label="安全库存" width="100" />
            <el-table-column prop="status" label="库存状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '需订购' ? 'danger' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { stockCalcApi } from '../api'

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
.page-container {
  height: 100%;
}
</style>
