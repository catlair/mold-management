<template>
  <div>
    <div class="tab-header">
      <el-button type="primary" size="small" @click="$emit('add')">{{ addLabel }}</el-button>
    </div>
    <el-table :data="paginatedData" border style="width: 100%">
      <slot />
    </el-table>
    <el-pagination
      v-model:current-page="currentPage"
      v-model:page-size="pageSize"
      :page-sizes="[10, 20, 50]"
      :total="data.length"
      layout="total, sizes, prev, pager, next"
      small
      style="margin-top: 12px; justify-content: flex-end;"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const props = defineProps<{
  data: any[]
  addLabel?: string
}>()

defineEmits<{ add: [] }>()

const currentPage = ref(1)
const pageSize = ref(10)

const paginatedData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return props.data.slice(start, start + pageSize.value)
})

watch(() => props.data, () => { currentPage.value = 1 })
</script>
