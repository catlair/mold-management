/**
 * 打印列配置：与屏幕表格解耦，独立维护打印所需的列元信息。
 * - field：行数据字段名（附件列用 _attachmentCount）
 * - label：屏幕全名（设置弹窗勾选列表展示）
 * - shortLabel：打印表头简写（屏幕表格列名保持不变）
 * - align：单元格对齐；numeric：标记为数字列触发 nowrap 防断行；format：可选单元格格式化
 */

export type PrintAlign = 'left' | 'center' | 'right'

export interface PrintColumn {
  field: string
  label: string
  shortLabel: string
  width?: string
  align?: PrintAlign
  numeric?: boolean
  format?: (row: Record<string, any>) => string
}

export function formatCell(row: Record<string, any>, column: PrintColumn): string {
  if (column.format) return column.format(row)
  const value = row?.[column.field]
  return value == null || value === '' ? '' : String(value)
}

export const screwSpecPrintColumns: PrintColumn[] = [
  { field: 'name', label: '螺丝名称', shortLabel: '名称', align: 'center', width: '18mm' },
  { field: 'headType', label: '头型', shortLabel: '头型', align: 'center', width: '9mm' },
  { field: 'punch', label: '冲头', shortLabel: '冲头', align: 'center', width: '13mm' },
  { field: 'threadType', label: '牙型', shortLabel: '牙型', align: 'center', width: '11mm' },
  { field: 'die', label: '牙板', shortLabel: '牙板', align: 'center', width: '13mm' },
  { field: 'headSize', label: '头/垫片大小', shortLabel: '头/垫', align: 'center', width: '10mm' },
  { field: 'headHeight', label: '头高', shortLabel: '头高', align: 'center', width: '14mm', numeric: true },
  { field: 'length', label: '长度', shortLabel: '长度', align: 'center', width: '13mm', numeric: true },
  { field: 'threadDiameter', label: '牙径', shortLabel: '牙径', align: 'center', width: '15mm', numeric: true },
  { field: 'shankLength', label: '光钉长度', shortLabel: '光钉', align: 'center', width: '13mm', numeric: true },
  { field: 'wireMaterial', label: '线材', shortLabel: '线材', align: 'center', width: '10mm' },
  { field: 'plating', label: '电镀', shortLabel: '电镀', align: 'center', width: '8mm' },
  { field: 'customer', label: '客户名', shortLabel: '客户', align: 'center', width: '11mm' },
  { field: 'externalId', label: '外部ID', shortLabel: '外ID', align: 'center', width: '9mm' },
  { field: 'remark', label: '备注', shortLabel: '备注', align: 'left', width: '8mm' },
  { field: '_attachmentCount', label: '附件', shortLabel: '附件', align: 'center', width: '11mm', format: row => String(row?._attachmentCount || 0) },
]

export const diePrintColumns: PrintColumn[] = [
  { field: 'name', label: '牙板名称', shortLabel: '名称', align: 'center', width: '26mm' },
  { field: 'machineType', label: '机型', shortLabel: '机型', align: 'center', width: '16mm' },
  { field: 'wireDiameter', label: '线径', shortLabel: '线径', align: 'center', width: '16mm', numeric: true },
  { field: 'safetyStock', label: '安全库存', shortLabel: '安全库存', align: 'center', width: '18mm', numeric: true },
  { field: 'currentStock', label: '当前库存', shortLabel: '当前库存', align: 'center', width: '18mm', numeric: true },
  { field: 'status', label: '库存状态', shortLabel: '状态', align: 'center', width: '16mm' },
  { field: 'remark', label: '备注', shortLabel: '备注', align: 'left', width: '24mm' },
]

export const punchPrintColumns: PrintColumn[] = [
  { field: 'name', label: '冲头名称', shortLabel: '名称', align: 'center', width: '26mm' },
  { field: 'spec', label: '规格', shortLabel: '规格', align: 'center', width: '14mm' },
  { field: 'material', label: '材质', shortLabel: '材质', align: 'center', width: '16mm' },
  { field: 'safetyStock', label: '安全库存', shortLabel: '安全库存', align: 'center', width: '18mm', numeric: true },
  { field: 'currentStock', label: '当前库存', shortLabel: '当前库存', align: 'center', width: '18mm', numeric: true },
  { field: 'status', label: '库存状态', shortLabel: '状态', align: 'center', width: '16mm' },
  { field: 'remark', label: '备注', shortLabel: '备注', align: 'left', width: '24mm' },
]

export const screwSpecPrintColumnFields = screwSpecPrintColumns.map(column => column.field)
export const diePrintColumnFields = diePrintColumns.map(column => column.field)
export const punchPrintColumnFields = punchPrintColumns.map(column => column.field)

/** 打印页面注册表：页面 key → 该页面可打印的列定义 */
export const PRINT_COLUMN_SETS: Record<string, PrintColumn[]> = {
  screwSpec: screwSpecPrintColumns,
  die: diePrintColumns,
  punch: punchPrintColumns,
}