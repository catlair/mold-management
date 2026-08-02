<template>
  <el-table
    ref="tableRef"
    :data="data"
    border
    :max-height="maxHeight"
    :fit="false"
    v-loading="loading"
  >
    <slot />
  </el-table>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'

const props = defineProps<{
  data: any[]
  loading?: boolean
}>()

const tableRef = ref<any>(null)
defineExpose({ tableRef })

const isFullscreen = inject('isFullscreen', ref(false))
const maxHeight = computed(() =>
  isFullscreen.value ? 'calc(100vh - 40px)' : 'calc(100vh - 184px)'
)
</script>
