/**
 * 共享表格过滤工具 —— 消除 7 个视图中重复的 filterHandler / statusFilters
 */

/** 到货状态筛选项（入库记录表通用） */
export const STATUS_FILTERS = [
  { label: '未到货', value: '未到货' },
  { label: '已到货', value: '已到货' },
]

/**
 * 精确匹配过滤器（vxe-table filter-method）
 * 用法: :filter-method="exactFilter"
 */
export function exactFilter({ value, row, column }: any): boolean {
  const property = column?.property ?? column?.field
  return row?.[property] === value
}

/**
 * 模糊匹配过滤器 —— 子串包含，大小写不敏感
 * 用法: :filter-method="fuzzyFilter"
 */
export function fuzzyFilter({ value, row, column }: any): boolean {
  const property = column?.property ?? column?.field
  const cell = row?.[property]
  if (cell == null || cell === '') return false
  return String(cell).toLowerCase().includes(String(value).toLowerCase())
}

/**
 * 从数据中提取唯一值，生成 vxe-table filters 选项
 * @param data  表格数据数组
 * @param field 字段名
 * @param opts  可选：{ labelFormatter } 自定义 label 显示
 */
export function buildFilters(
  data: any[],
  field: string,
  opts?: { labelFormatter?: (v: any) => string },
): Array<{ label: string; value: any }> {
  const seen = new Set<string>()
  const result: Array<{ label: string; value: any }> = []
  for (const row of data) {
    const v = row?.[field]
    if (v === '' || v == null) continue
    const key = String(v)
    if (seen.has(key)) continue
    seen.add(key)
    result.push({
      label: opts?.labelFormatter ? opts.labelFormatter(v) : String(v),
      value: v,
    })
  }
  return result
}
