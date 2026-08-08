<template>
  <Teleport to="body">
    <div
      v-if="paginatedPages.length && currentPageKey === pageKey"
      class="print-area"
      :class="[
        settings.portrait ? 'is-portrait' : 'is-landscape',
        `print-style-${settings.styleId}`,
        { 'has-striped-rows': settings.striped },
      ]"
      :style="{
        '--print-font-family': settings.fontFamily,
        '--print-font-size': `${settings.fontSize}pt`,
        '--print-stripe-color': settings.stripeColor,
      }"
      aria-hidden="true"
    >
      <div v-for="(page, index) in paginatedPages" :key="index" class="print-page">
        <header class="print-header">
          <span class="ph-title">{{ title }}</span>
          <span class="ph-meta">模具管理系统 · 打印时间 {{ printTime }} · 共 {{ paginatedPages.length }} 页 / {{ rows.length }} 行</span>
        </header>
        <table class="print-table">
          <colgroup>
            <col v-for="column in enabledColumns" :key="column.field" :style="{ width: column.width || 'auto' }" />
          </colgroup>
          <thead>
            <tr>
              <th v-for="column in enabledColumns" :key="column.field">{{ column.shortLabel }}</th>
            </tr>
          </thead>
          <tbody>
            <template v-if="page.groups && page.groups.length">
              <template v-for="(group, gIndex) in page.groups" :key="`group-${index}-${gIndex}`">
                <tr class="print-group-header">
                  <td :colspan="enabledColumns.length" class="align-left">
                    <span class="pg-customer">客户：{{ group.customer }}</span>
                    <span class="pg-count">共 {{ group.rows.length }} 条</span>
                  </td>
                </tr>
                <tr
                  v-for="(row, rowIndex) in group.rows"
                  :key="row.id ?? `${index}-${gIndex}-${rowIndex}`"
                  class="print-data-row"
                  :class="{ 'is-stripe-row': (page.startRowIndex + groupOffsets(page)[gIndex] + rowIndex) % 2 === 1 }"
                >
                  <td
                    v-for="column in enabledColumns"
                    :key="column.field"
                    :class="[`align-${column.align || 'center'}`, { 'is-numeric': column.numeric }]"
                  >
                    <template v-if="column.tolerance && toleranceView(row, column).isAsymmetric">
                      <span class="print-tolerance" :title="toleranceView(row, column).fullText">
                        <span v-if="toleranceView(row, column).prefix" class="pt-affix">{{ toleranceView(row, column).prefix }}</span>
                        <span class="pt-nominal">{{ toleranceView(row, column).nominal }}</span>
                        <span class="pt-deviations">
                          <span class="pt-upper">{{ toleranceView(row, column).upper }}</span>
                          <span class="pt-lower">{{ toleranceView(row, column).lower }}</span>
                        </span>
                        <span v-if="toleranceView(row, column).suffix" class="pt-affix">{{ toleranceView(row, column).suffix }}</span>
                      </span>
                    </template>
                    <template v-else>{{ formatCell(row, column) }}</template>
                  </td>
                </tr>
              </template>
            </template>
            <template v-else>
              <tr
                v-for="(row, rowIndex) in page.rows"
                :key="row.id ?? `${index}-${rowIndex}`"
                class="print-data-row"
                :class="{ 'is-stripe-row': (page.startRowIndex + rowIndex) % 2 === 1 }"
              >
                <td
                  v-for="column in enabledColumns"
                  :key="column.field"
                  :class="[`align-${column.align || 'center'}`, { 'is-numeric': column.numeric }]"
                >
                  {{ formatCell(row, column) }}
                </td>
              </tr>
            </template>
            <tr
              v-for="fillerIndex in page.fillerRows"
              :key="`filler-${index}-${fillerIndex}`"
              class="print-filler-row"
              :class="{ 'is-stripe-row': (page.startRowIndex + page.rows.length + fillerIndex - 1) % 2 === 1 }"
              :style="{ height: `${page.fillerRowHeightMm}mm` }"
              aria-hidden="true"
            >
              <td
                v-for="column in enabledColumns"
                :key="column.field"
                :style="{ height: `${page.fillerRowHeightMm}mm` }"
              ><span class="print-filler-cell">&nbsp;</span></td>
            </tr>
          </tbody>
        </table>
        <footer class="print-footer">
          <span>第 {{ index + 1 }} 页 / 共 {{ paginatedPages.length }} 页</span>
          <span>{{ printTime }}</span>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { formatCell, type PrintColumn } from '../config/printColumns'
import type { PrintPageData } from '../composables/usePrint'
import type { PrintPageSettings } from '../composables/usePrintSettings'
import { buildToleranceDisplay } from '../utils/toleranceDisplay'

defineProps<{
  pageKey: string
  currentPageKey: string
  rows: any[]
  title: string
  printTime: string
  settings: PrintPageSettings
  enabledColumns: PrintColumn[]
  paginatedPages: PrintPageData[]
}>()

function toleranceView(row: Record<string, any>, column: PrintColumn) {
  return buildToleranceDisplay(formatCell(row, column))
}

/** 每页各组的数据行起始偏移（用于斑马纹跨组连续着色） */
function groupOffsets(page: PrintPageData): number[] {
  const offsets: number[] = []
  let acc = 0
  for (const group of page.groups ?? []) {
    offsets.push(acc)
    acc += group.rows.length
  }
  return offsets
}
</script>
