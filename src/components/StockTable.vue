<template>
  <div class="stock-table-wrapper">
    <el-table :data="data" border style="width: 100%" v-loading="loading">
      <el-table-column prop="name" :label="nameLabel" min-width="150" sortable />
      <el-table-column prop="currentStock" label="当前库存" width="120" sortable />
      <el-table-column prop="safetyStock" label="安全库存" width="120" sortable />
      <el-table-column prop="status" label="库存状态" width="120" sortable>
        <template #default="{ row }">
          <el-tag :type="row.status === '需订购' ? 'danger' : 'success'" effect="dark" round>
            {{ row.status }}
          </el-tag>
        </template>
      </el-table-column>
      <template #empty>
        <el-empty description="暂无库存数据" />
      </template>
    </el-table>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  data: any[]
  nameLabel: string
  loading?: boolean
}>()
</script>

<style scoped>
.stock-table-wrapper {
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
