<template>
  <span v-if="view.isAsymmetric" class="tolerance-display" :title="view.fullText">
    <span v-if="view.prefix" class="tolerance-affix">{{ view.prefix }}</span>
    <span class="tolerance-nominal">{{ view.nominal }}</span>
    <span class="tolerance-deviations" aria-label="上下偏差">
      <span class="tolerance-upper">{{ view.upper }}</span>
      <span class="tolerance-lower">{{ view.lower }}</span>
    </span>
    <span v-if="view.suffix" class="tolerance-affix">{{ view.suffix }}</span>
  </span>
  <span v-else>{{ view.fullText || emptyText }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { buildToleranceDisplay } from '../utils/toleranceDisplay'

const props = withDefaults(defineProps<{
  value?: string | number | null
  emptyText?: string
}>(), {
  value: '',
  emptyText: '-',
})

const view = computed(() => buildToleranceDisplay(props.value))
</script>

<style scoped>
.tolerance-display {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 2px;
  min-height: 32px;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}
.tolerance-nominal { line-height: 1; }
.tolerance-deviations { display: inline-grid; grid-template-rows: 1fr 1fr; align-items: center; margin-left: 1px; line-height: 1; vertical-align: middle; }
.tolerance-upper,
.tolerance-lower { display: block; padding: 0 1px; font-size: .72em; line-height: 1.05; text-align: left; }
.tolerance-upper { align-self: end; }
.tolerance-lower { align-self: start; }
.tolerance-affix { margin: 0 1px; }
</style>
