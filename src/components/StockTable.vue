<template>
  <vxe-table
    :data="data"
    border
    round
    stripe
    show-overflow="tooltip"
    style="width: 100%"
    :loading="loading"
  >
    <vxe-column field="name" :title="nameLabel" min-width="150" sortable />
    <vxe-column field="currentStock" title="当前库存" width="120" sortable />
    <vxe-column field="safetyStock" title="安全库存" width="120" sortable />
    <vxe-column field="status" title="库存状态" width="120" sortable>
      <template #default="{ row }">
        <el-tag :type="row.status === '需订购' ? 'danger' : 'success'" effect="dark" round>
          {{ row.status }}
        </el-tag>
      </template>
    </vxe-column>
    <template #empty>
      <el-empty description="暂无库存数据" />
    </template>
  </vxe-table>
</template>

<script setup lang="ts">
defineProps<{
  data: any[]
  nameLabel: string
  loading?: boolean
}>()
</script>