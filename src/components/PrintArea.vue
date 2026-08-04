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
import type { PrintSettings } from '../composables/usePrintSettings'

defineProps<{
  pageKey: string
  currentPageKey: string
  rows: any[]
  title: string
  printTime: string
  settings: PrintSettings
  enabledColumns: PrintColumn[]
  paginatedPages: PrintPageData[]
}>()
</script>
