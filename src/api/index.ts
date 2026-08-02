import { invoke } from '@tauri-apps/api/core'

// ========== 浏览器预览模式 ==========
// 在纯浏览器环境（无 Tauri 后端）提供 mock 数据，便于开发/自测/演示。
// Tauri 桌面应用内走真实 invoke，不受影响。
const isTauri = () =>
  typeof window !== 'undefined' &&
  '__TAURI_INTERNALS__' in (window as any)

function generateMockRows(sheetName: string): any[] {
  const count = sheetName.includes('信息') ? 60 : 25
  const rows: any[] = []
  for (let i = 1; i <= count; i++) {
    const id = `mock_${i}`
    if (sheetName.includes('螺丝规格')) {
      rows.push({
        id, name: `MOCK螺丝-${i}`, headType: '盘头', punch: `P${100 + i}`,
        threadType: '自攻', die: `D${200 + i}`, headSize: '10.5', headHeight: '3.5',
        length: `${20 + (i % 20)}`, threadDiameter: '4.2', shankLength: '8',
        wireMaterial: 'SWRCH22A', plating: '镀镍', customer: `客户${i % 5}`,
        externalId: `EXT-${i}`, remark: `mock 数据第 ${i} 行`
      })
    } else if (sheetName.includes('冲头信息')) {
      rows.push({ id, name: `MOCK冲头-${i}`, spec: `规格${i}`, material: 'SKD11', safetyStock: 10 + (i % 5), remark: `mock ${i}` })
    } else if (sheetName.includes('牙板信息')) {
      rows.push({ id, name: `MOCK牙板-${i}`, machineType: `机型${i % 3}`, wireDiameter: `${(i % 20) + 1}`, safetyStock: 20 + (i % 8), remark: `mock ${i}` })
    } else if (sheetName.includes('皮带信息')) {
      rows.push({ id, name: `MOCK皮带-${i}`, spec: `带宽${i}`, machine: `机器${i % 4}`, safetyStock: 5, remark: `mock ${i}` })
    } else if (sheetName.includes('主模具信息')) {
      rows.push({ id, name: `MOCK主模-${i}`, holeDiameter: `${i % 30}`, wireMaterial: 'SWRCH22A', safetyStock: 8, remark: `mock ${i}` })
    } else if (sheetName.includes('剪刀信息')) {
      rows.push({ id, name: `MOCK剪刀-${i}`, diameter: `${i % 25}`, wireMaterial: 'SWRCH22A', safetyStock: 6, remark: `mock ${i}` })
    } else if (sheetName.includes('上冲信息')) {
      rows.push({ id, name: `MOCK上冲-${i}`, diameter: `${i % 25}`, wireMaterial: 'SWRCH22A', safetyStock: 6, remark: `mock ${i}` })
    } else {
      rows.push({ id, name: `MOCK-${i}`, remark: `mock ${i}` })
    }
  }
  return rows
}

function createMockApi(sheetName: string) {
  const rows = generateMockRows(sheetName)
  return {
    getAll: async () => [...rows],
    getById: async (id: string) => rows.find(r => r.id === id) || null,
    add: async (data: any) => ({ id: `mock_new_${Date.now()}`, ...data }),
    update: async (id: string, data: any) => ({ id, ...data }),
    remove: async () => true,
  }
}

// 创建通用 CRUD API (Tauri invoke；浏览器预览模式返回 mock 数据)
export function createApi(sheetName: string) {
  if (!isTauri()) {
    return createMockApi(sheetName)
  }
  return {
    getAll: () => invoke<any[]>('get_all_records', { sheetName }),
    getById: (id: string) => invoke<any>('get_record', { sheetName, id }),
    add: (data: any) => invoke<any>('add_record', { sheetName, item: data }),
    update: (id: string, data: any) => invoke<any>('update_record', { sheetName, id, data }),
    remove: (id: string) => invoke<boolean>('delete_record', { sheetName, id }),
  }
}

// 表名常量
export const SHEETS = {
  SCREW_SPEC: '螺丝规格表',
  PUNCH_INFO: '冲头信息表',
  PUNCH_ORDER: '冲头入库记录',
  PUNCH_USE: '冲头领用记录',
  PUNCH_LINK: '冲头-螺丝规格关联',
  PUNCH_STOCK: '冲头库存汇总',
  DIE_INFO: '牙板信息表',
  DIE_ORDER: '牙板入库记录',
  DIE_USE: '牙板领用记录',
  DIE_LINK: '牙板-螺丝规格关联',
  DIE_STOCK: '牙板库存汇总',
  BELT_INFO: '皮带信息表',
  BELT_ORDER: '皮带入库记录',
  BELT_USE: '皮带使用记录',
  BELT_STOCK: '皮带库存汇总',
  MAIN_MOLD_INFO: '主模具信息表',
  MAIN_MOLD_ORDER: '主模具入库记录',
  MAIN_MOLD_USE: '主模具使用记录',
  MAIN_MOLD_LINK: '主模具-线材关联',
  MAIN_MOLD_STOCK: '主模具库存汇总',
  SCISSOR_INFO: '剪刀信息表',
  SCISSOR_ORDER: '剪刀入库记录',
  SCISSOR_USE: '剪刀使用记录',
  SCISSOR_LINK: '剪刀-线材关联',
  SCISSOR_STOCK: '剪刀库存汇总',
  UPPER_PUNCH_INFO: '上冲信息表',
  UPPER_PUNCH_ORDER: '上冲入库记录',
  UPPER_PUNCH_USE: '上冲使用记录',
  UPPER_PUNCH_LINK: '上冲-线材关联',
  UPPER_PUNCH_STOCK: '上冲库存汇总'
}

// 各模块 API
export const screwSpecApi = createApi(SHEETS.SCREW_SPEC)

export const punchApi = createApi(SHEETS.PUNCH_INFO)
export const punchOrderApi = createApi(SHEETS.PUNCH_ORDER)
export const punchUseApi = createApi(SHEETS.PUNCH_USE)
export const punchLinkApi = createApi(SHEETS.PUNCH_LINK)
export const punchStockApi = createApi(SHEETS.PUNCH_STOCK)

export const dieApi = createApi(SHEETS.DIE_INFO)
export const dieOrderApi = createApi(SHEETS.DIE_ORDER)
export const dieUseApi = createApi(SHEETS.DIE_USE)
export const dieLinkApi = createApi(SHEETS.DIE_LINK)
export const dieStockApi = createApi(SHEETS.DIE_STOCK)

export const beltApi = createApi(SHEETS.BELT_INFO)
export const beltOrderApi = createApi(SHEETS.BELT_ORDER)
export const beltUseApi = createApi(SHEETS.BELT_USE)
export const beltStockApi = createApi(SHEETS.BELT_STOCK)

export const mainMoldApi = createApi(SHEETS.MAIN_MOLD_INFO)
export const mainMoldOrderApi = createApi(SHEETS.MAIN_MOLD_ORDER)
export const mainMoldUseApi = createApi(SHEETS.MAIN_MOLD_USE)
export const mainMoldLinkApi = createApi(SHEETS.MAIN_MOLD_LINK)
export const mainMoldStockApi = createApi(SHEETS.MAIN_MOLD_STOCK)

export const scissorApi = createApi(SHEETS.SCISSOR_INFO)
export const scissorOrderApi = createApi(SHEETS.SCISSOR_ORDER)
export const scissorUseApi = createApi(SHEETS.SCISSOR_USE)
export const scissorLinkApi = createApi(SHEETS.SCISSOR_LINK)
export const scissorStockApi = createApi(SHEETS.SCISSOR_STOCK)

export const upperPunchApi = createApi(SHEETS.UPPER_PUNCH_INFO)
export const upperPunchOrderApi = createApi(SHEETS.UPPER_PUNCH_ORDER)
export const upperPunchUseApi = createApi(SHEETS.UPPER_PUNCH_USE)
export const upperPunchLinkApi = createApi(SHEETS.UPPER_PUNCH_LINK)
export const upperPunchStockApi = createApi(SHEETS.UPPER_PUNCH_STOCK)

// 库存计算 API（浏览器预览模式返回 mock）
export const stockCalcApi = isTauri()
  ? {
      calculateAll: () => invoke<Record<string, any[]>>('calculate_stock', { stockType: 'all' }),
      calculate: (type: string) => invoke<any[]>('calculate_stock', { stockType: type }),
    }
  : {
      calculateAll: async () => ({} as Record<string, any[]>),
      calculate: async () => [] as any[],
    }

// 数据导入导出 API（浏览器预览模式返回 mock）
export const dataApi = isTauri()
  ? {
      exportData: () => invoke<any>('export_data'),
      importData: (base64: string) => invoke<{ success: boolean; stats: Record<string, number> }>('import_data', { data: base64 }),
    }
  : {
      exportData: async () => ({ filename: 'mold-data.xlsx', data: '' }),
      importData: async () => ({ success: true, stats: {} }),
    }

// 配置 API（浏览器预览模式返回 mock）
export const settingsApi = isTauri()
  ? {
      getDataPath: () => invoke<string>('get_file_path_cmd'),
      setDataPath: (path: string) => invoke<{ success: boolean; filePath: string }>('set_file_path', { path }),
    }
  : {
      getDataPath: async () => '（浏览器预览模式，无真实数据文件）',
      setDataPath: async () => ({ success: true, filePath: '' }),
    }

// 备份 API（浏览器预览模式返回 mock）
export const backupApi = isTauri()
  ? {
      backup: () => invoke<any>('backup_data'),
      getConfig: () => invoke<any>('get_backup_config'),
      setConfig: (backupCount: number, backupPath: string | null) => invoke<any>('set_backup_config', { backupCount, backupPath }),
      list: () => invoke<any[]>('list_backups'),
      toggleLock: (index: number) => invoke<{ success: boolean; locked: boolean }>('toggle_backup_lock', { index }),
      restore: (backupPath: string) => invoke<{ success: boolean }>('restore_backup', { backupPath }),
    }
  : {
      backup: async () => ({ success: true }),
      getConfig: async () => ({ backupCount: 10, backupPath: null, defaultBackupDir: '', effectiveBackupDir: '' }),
      setConfig: async () => ({ success: true }),
      list: async () => [] as any[],
      toggleLock: async () => ({ success: true, locked: false }),
      restore: async () => ({ success: true }),
    }

// 删除权限 API（浏览器预览模式返回 mock）
export const allowDeleteApi = isTauri()
  ? {
      get: () => invoke<boolean>('get_allow_delete'),
      set: (allow: boolean) => invoke<any>('set_allow_delete', { allow }),
    }
  : {
      get: async () => true,
      set: async () => ({ success: true }),
    }
