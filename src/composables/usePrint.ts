/**
 * 打印核心：分页（按实际行高测量切分）+ 系统打印 + 导出 PDF。
 * - print()：写入打印数据 → 动态注入 @page 方向 → window.print()（同步模态，打印后原地返回）
 * - exportPdf()：html2canvas 逐页截图 + jsPDF 拼接导出
 * - 分页：先用隐藏 probe 渲染整表测量每行高度，再按 A4 可用高度动态切页，
 *   避免固定行数在长文本换行时把行截断
 */
import { ref, nextTick } from 'vue'
import { jsPDF } from 'jspdf'
import html2canvas from 'html2canvas'
import { ElMessage } from 'element-plus'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { isTauriEnvironment } from '../api'
import { isUserCancellation } from '../utils/errorFeedback'
import { usePrintSettings } from './usePrintSettings'
import { formatCell, type PrintColumn } from '../config/printColumns'

const MM_PER_PX = 0.264583
const PAGE_PADDING_MM = 20
const TABLE_TOP_GAP_MM = 3
const PAGE_SAFETY_MM = 1

export interface PrintPageData {
  rows: any[]
  startRowIndex: number
  fillerRows: number
  fillerRowHeightMm: number
}

const rows = ref<any[]>([])
const title = ref('')
const printTime = ref('')
const isRendering = ref(false)
const paginatedPages = ref<PrintPageData[]>([])
const enabledColumns = ref<PrintColumn[]>([])

function formatNow(): string {
  const d = new Date()
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`
}

function fileStamp(value: string): string {
  return value.replace(/[: ]/g, '-')
}

function applyPageSize(portrait: boolean) {
  let el = document.getElementById('print-page-size') as HTMLStyleElement | null
  if (!el) {
    el = document.createElement('style')
    el.id = 'print-page-size'
    document.head.appendChild(el)
  }
  el.textContent = `@page { size: A4 ${portrait ? 'portrait' : 'landscape'}; margin: 0; }`
}

function prepareArea() {
  let area = document.querySelector<HTMLElement>('.print-area')
  if (!area) {
    area = document.createElement('div')
    area.className = 'print-area'
    document.body.appendChild(area)
  }
  return area
}

/**
 * 用隐藏 probe 渲染整表，测量表头与每行高度（px），
 * 然后按 A4 可用高度（mm）动态切分，行高自适应长文本换行。
 */
function measureAndPaginate(
  data: any[],
  columns: PrintColumn[],
  settings: { portrait: boolean; fontFamily: string; fontSize: number },
): PrintPageData[] {
  if (!data.length) return []

  const probe = document.createElement('div')
  probe.className = 'print-probe'
  probe.style.cssText = `position: fixed; left: -99999px; top: 0; width: ${settings.portrait ? 210 : 297}mm; padding: 10mm; box-sizing: border-box; background: #fff; color: #000;`
  probe.style.fontFamily = settings.fontFamily
  probe.style.fontSize = `${settings.fontSize}pt`
  probe.style.lineHeight = '1.35'

  const header = document.createElement('header')
  header.className = 'print-header'
  const headerTitle = document.createElement('span')
  headerTitle.className = 'ph-title'
  headerTitle.textContent = '打印标题'
  const headerMeta = document.createElement('span')
  headerMeta.className = 'ph-meta'
  headerMeta.textContent = '模具管理系统 · 打印时间 0000-00-00 00:00:00 · 共 1 页 / 0 行'
  header.append(headerTitle, headerMeta)
  probe.appendChild(header)

  const table = document.createElement('table')
  table.className = 'print-table'

  const colgroup = document.createElement('colgroup')
  columns.forEach((column) => {
    const col = document.createElement('col')
    if (column.width) col.style.width = column.width
    colgroup.appendChild(col)
  })
  table.appendChild(colgroup)

  const thead = document.createElement('thead')
  const headerRow = document.createElement('tr')
  columns.forEach((column) => {
    const th = document.createElement('th')
    th.textContent = column.shortLabel
    headerRow.appendChild(th)
  })
  thead.appendChild(headerRow)
  table.appendChild(thead)

  const tbody = document.createElement('tbody')
  data.forEach((row) => {
    const tr = document.createElement('tr')
    columns.forEach((column) => {
      const td = document.createElement('td')
      if (column.numeric) td.className = 'is-numeric'
      td.textContent = formatCell(row, column)
      tr.appendChild(td)
    })
    tbody.appendChild(tr)
  })
  table.appendChild(tbody)

  probe.appendChild(table)
  const footer = document.createElement('footer')
  footer.className = 'print-footer print-probe-footer'
  footer.innerHTML = '<span>第 1 页 / 共 1 页</span><span>0000-00-00 00:00:00</span>'
  probe.appendChild(footer)
  document.body.appendChild(probe)

  const headerMm = header.offsetHeight * MM_PER_PX
  const tableHeaderPx = thead.offsetHeight || 24
  const footerMm = footer.offsetHeight * MM_PER_PX
  const rowHeightsPx = Array.from(tbody.querySelectorAll('tr')).map(tr => tr.offsetHeight || 14)
  document.body.removeChild(probe)

  const pageHeightMm = settings.portrait ? 297 : 210
  const tableHeaderMm = tableHeaderPx * MM_PER_PX
  const bodyAvailableMm = Math.max(
    0,
    pageHeightMm
      - PAGE_PADDING_MM
      - headerMm
      - footerMm
      - TABLE_TOP_GAP_MM
      - tableHeaderMm
      - PAGE_SAFETY_MM,
  )
  const measuredRowMm = rowHeightsPx.map(heightPx => Math.max(heightPx, 14) * MM_PER_PX)
  const fillerRowMm = Math.max(
    Math.min(...measuredRowMm.filter(heightMm => Number.isFinite(heightMm))),
    14 * MM_PER_PX,
  )

  const pages: Array<{ rows: any[]; usedBodyMm: number }> = []
  let currentRows: any[] = []
  let usedBodyMm = 0

  for (let i = 0; i < data.length; i++) {
    const rowMm = measuredRowMm[i] || fillerRowMm
    if (usedBodyMm + rowMm > bodyAvailableMm && currentRows.length) {
      pages.push({ rows: currentRows, usedBodyMm })
      currentRows = [data[i]]
      usedBodyMm = rowMm
    } else {
      currentRows.push(data[i])
      usedBodyMm += rowMm
    }
  }
  if (currentRows.length) pages.push({ rows: currentRows, usedBodyMm })

  let startRowIndex = 0
  return pages.map((page) => {
    const remainingMm = Math.max(0, bodyAvailableMm - page.usedBodyMm)
    const fillerRows = Math.max(0, Math.floor(remainingMm / fillerRowMm))
    const result = {
      rows: page.rows,
      startRowIndex,
      fillerRows,
      fillerRowHeightMm: fillerRows ? remainingMm / fillerRows : 0,
    }
    startRowIndex += page.rows.length
    return result
  })
}

export function usePrint() {
  const { settings, getEnabledFields } = usePrintSettings()
  const currentPageKey = ref('')

  async function prepareState(
    pageKey: string,
    data: any[],
    columns: PrintColumn[],
    opts: { title?: string } = {},
  ) {
    rows.value = data
    title.value = opts.title || '数据明细表'
    printTime.value = formatNow()
    currentPageKey.value = pageKey
    const enabledFields = getEnabledFields(pageKey)
    enabledColumns.value = columns.filter(column => enabledFields.includes(column.field))
    await nextTick()
    paginatedPages.value = measureAndPaginate(data, enabledColumns.value, settings)
  }

  function print(pageKey: string, data: any[], columns: PrintColumn[], opts: { title?: string } = {}) {
    if (!data.length) {
      ElMessage.warning('没有可打印的数据')
      return
    }
    void prepareState(pageKey, data, columns, opts).then(() => {
      applyPageSize(settings.portrait)
      requestAnimationFrame(() => window.print())
    })
  }

  async function exportPdf(pageKey: string, data: any[], columns: PrintColumn[], opts: { title?: string } = {}) {
    if (!data.length) {
      ElMessage.warning('没有可导出的数据')
      return
    }
    await prepareState(pageKey, data, columns, opts)
    isRendering.value = true
    await nextTick()
    const area = prepareArea()
    area.classList.add('is-rendering')
    try {
      const pageEls = Array.from(area.querySelectorAll<HTMLElement>('.print-page'))
      if (!pageEls.length) {
        ElMessage.warning('没有可导出的页面')
        return
      }
      const pdf = new jsPDF({
        orientation: settings.portrait ? 'portrait' : 'landscape',
        unit: 'mm',
        format: 'a4',
      })
      const pageWidth = settings.portrait ? 210 : 297
      const pageHeight = settings.portrait ? 297 : 210
      for (let i = 0; i < pageEls.length; i++) {
        pageEls.forEach((el, index) => {
          el.classList.toggle('is-capture', i === index)
          el.classList.toggle('is-hidden', i !== index)
        })
        await nextTick()
        const canvas = await html2canvas(pageEls[i], {
          scale: 2,
          backgroundColor: '#ffffff',
          useCORS: true,
          logging: false,
        })
        const image = canvas.toDataURL('image/jpeg', 0.95)
        if (i > 0) pdf.addPage()
        pdf.addImage(image, 'JPEG', 0, 0, pageWidth, pageHeight)
      }
      const filename = `${title.value}-${fileStamp(printTime.value)}.pdf`
      if (isTauriEnvironment()) {
        // Tauri 桌面环境：原生保存对话框 + 直接写文件（WebView2 不支持浏览器下载弹窗）
        const filePath = await save({
          defaultPath: filename,
          filters: [{ name: 'PDF 文件', extensions: ['pdf'] }],
        })
        if (!filePath) return
        const bytes = new Uint8Array(pdf.output('arraybuffer'))
        await writeFile(filePath, bytes)
        ElMessage.success('PDF 导出成功')
      } else {
        // 纯浏览器预览模式：回退为浏览器下载
        pdf.save(filename)
      }
    } catch (error) {
      if (isUserCancellation(error)) return
      console.error('导出 PDF 失败', error)
      ElMessage.error('导出 PDF 失败，请重试')
    } finally {
      area.classList.remove('is-rendering')
      isRendering.value = false
    }
  }

  return {
    rows,
    title,
    printTime,
    isRendering,
    currentPageKey,
    paginatedPages,
    enabledColumns,
    print,
    exportPdf,
  }
}
