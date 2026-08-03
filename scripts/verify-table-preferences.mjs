import { readFile } from 'node:fs/promises'

const source = await readFile(new URL('../src/composables/useTablePreferences.ts', import.meta.url), 'utf8')
const required = [
  "kind: 'mold-management-table-preferences'",
  'version: 1',
  'createTablePreferencesExport',
  'parseTablePreferencesImport',
  'applyTablePreferencesImport',
  'ignoredTableCount',
  'ignoredColumnCount',
  'window.localStorage.setItem(STORAGE_KEY, serialized)',
]
const missing = required.filter(token => !source.includes(token))
if (missing.length) {
  console.error(`配置导入导出实现缺少关键项：${missing.join(', ')}`)
  process.exit(1)
}
console.log('配置导入导出结构检查通过')
