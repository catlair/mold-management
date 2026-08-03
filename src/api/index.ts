import { invoke } from '@tauri-apps/api/core'
import { dieUniqueKey, punchUniqueKey } from '../utils/duplicateDetection'

export type AttachmentKind = 'image' | 'pdf'
export type AnnotationTool = 'pen' | 'rectangle' | 'arrow' | 'text'

export interface AttachmentAnnotation {
  id: string
  page: number
  tool: AnnotationTool
  color: string
  strokeWidth: number
  x: number
  y: number
  endX?: number
  endY?: number
  text?: string
  points?: Array<{ x: number; y: number }>
}

export interface ScrewAttachment {
  id: string
  screwSpecId: string
  displayName: string
  fileName: string
  mimeType: string
  size: number
  relativePath: string
  annotations: AttachmentAnnotation[]
  sortOrder: number
  createdAt: string
  updatedAt: string
}

export interface AttachmentContent {
  attachment: ScrewAttachment
  data: string
}

// ========== 浏览器预览模式 ==========
// 在纯浏览器环境（无 Tauri 后端）提供 mock 数据，便于开发/自测/演示。
// Tauri 桌面应用内走真实 invoke，不受影响。
export const isTauriEnvironment = () =>
  typeof window !== 'undefined' &&
  '__TAURI_INTERNALS__' in (window as any)

const isTauri = isTauriEnvironment

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

function getMockUniqueKey(sheetName: string, row: any): string | null {
  if (sheetName === '冲头信息表') return punchUniqueKey(row)
  if (sheetName === '牙板信息表') return dieUniqueKey(row)
  return null
}

function createMockApi(sheetName: string) {
  const rows = generateMockRows(sheetName)

  function ensureMockUnique(data: any, excludeId = '') {
    const candidateKey = getMockUniqueKey(sheetName, data)
    if (!candidateKey) return
    const existing = rows.find(row => row.id !== excludeId && getMockUniqueKey(sheetName, row) === candidateKey)
    if (existing) {
      const resource = sheetName === '冲头信息表' ? '冲头' : '牙板'
      throw new Error(`DUPLICATE_RECORD|${resource}|${existing.id}`)
    }
  }

  return {
    getAll: async () => rows.map(row => ({ ...row })),
    getById: async (id: string) => rows.find(row => row.id === id) || null,
    add: async (data: any) => {
      ensureMockUnique(data)
      const created = { ...data, id: `mock_new_${Date.now()}_${rows.length}` }
      rows.push(created)
      return { ...created }
    },
    update: async (id: string, data: any) => {
      const currentIndex = rows.findIndex(row => row.id === id)
      if (currentIndex < 0) throw new Error('记录未找到')
      const current = rows[currentIndex]
      const updated = { ...current, ...data, id }
      if (getMockUniqueKey(sheetName, current) !== getMockUniqueKey(sheetName, updated)) {
        ensureMockUnique(updated, id)
      }
      rows[currentIndex] = updated
      return { ...updated }
    },
    remove: async (id: string) => {
      const index = rows.findIndex(row => row.id === id)
      if (index < 0) return false
      rows.splice(index, 1)
      return true
    },
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

const mockAttachments = new Map<string, ScrewAttachment[]>()
const mockAttachmentData = new Map<string, string>()
const MOCK_ATTACHMENT_STORAGE_KEY = 'mold-management-mock-attachments'
let mockAttachmentsInitialized = false

function persistMockAttachments() {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(
    MOCK_ATTACHMENT_STORAGE_KEY,
    JSON.stringify(Object.fromEntries(mockAttachments.entries()))
  )
}

function bytesToBase64(bytes: Uint8Array) {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary)
}

function createMockImageData() {
  const canvas = document.createElement('canvas')
  canvas.width = 1200
  canvas.height = 760
  const context = canvas.getContext('2d')!
  context.fillStyle = '#f8fafc'
  context.fillRect(0, 0, canvas.width, canvas.height)
  context.fillStyle = '#0f172a'
  context.font = '700 46px "Microsoft YaHei", sans-serif'
  context.fillText('螺丝规格附件示意图', 72, 92)
  context.strokeStyle = '#2563eb'
  context.lineWidth = 8
  context.strokeRect(160, 210, 880, 300)
  context.beginPath()
  context.moveTo(230, 360)
  context.lineTo(970, 360)
  context.stroke()
  context.fillStyle = '#475569'
  context.font = '30px "Microsoft YaHei", sans-serif'
  context.fillText('浏览器演示附件，可直接添加标注并保存', 250, 600)
  return canvas.toDataURL('image/png').split(',')[1]
}

function createMockPdfData() {
  const objects = [
    '1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n',
    '2 0 obj\n<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>\nendobj\n',
    '3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 6 0 R >>\nendobj\n',
    '4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n',
    '5 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 7 0 R >>\nendobj\n',
    '6 0 obj\n<< /Length 68 >>\nstream\nBT /F1 24 Tf 72 700 Td (Screw Specification - Page 1) Tj ET\nendstream\nendobj\n',
    '7 0 obj\n<< /Length 68 >>\nstream\nBT /F1 24 Tf 72 700 Td (Screw Specification - Page 2) Tj ET\nendstream\nendobj\n',
  ]
  let pdf = '%PDF-1.4\n'
  const offsets = [0]
  for (const object of objects) {
    offsets.push(new TextEncoder().encode(pdf).length)
    pdf += object
  }
  const xrefOffset = new TextEncoder().encode(pdf).length
  pdf += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`
  for (let index = 1; index <= objects.length; index++) {
    pdf += `${String(offsets[index]).padStart(10, '0')} 00000 n \n`
  }
  pdf += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF`
  return bytesToBase64(new TextEncoder().encode(pdf))
}

function initializeMockAttachments() {
  if (mockAttachmentsInitialized || typeof document === 'undefined') return
  mockAttachmentsInitialized = true
  const now = new Date().toISOString()
  const items: ScrewAttachment[] = [
    {
      id: 'mock_attachment_image', screwSpecId: 'mock_1', displayName: '头部尺寸示意图.png',
      fileName: '头部尺寸示意图.png', mimeType: 'image/png', size: 18420,
      relativePath: 'mock/diagram.png', annotations: [], sortOrder: 0,
      createdAt: now, updatedAt: now,
    },
    {
      id: 'mock_attachment_pdf', screwSpecId: 'mock_1', displayName: '客户规格确认书.pdf',
      fileName: '客户规格确认书.pdf', mimeType: 'application/pdf', size: 2048,
      relativePath: 'mock/specification.pdf', annotations: [], sortOrder: 1,
      createdAt: now, updatedAt: now,
    },
  ]
  const stored = localStorage.getItem(MOCK_ATTACHMENT_STORAGE_KEY)
  if (stored) {
    try {
      const parsed = JSON.parse(stored) as Record<string, ScrewAttachment[]>
      for (const [screwSpecId, storedItems] of Object.entries(parsed)) {
        mockAttachments.set(screwSpecId, storedItems)
      }
    } catch {
      localStorage.removeItem(MOCK_ATTACHMENT_STORAGE_KEY)
    }
  }
  if (!mockAttachments.has('mock_1')) mockAttachments.set('mock_1', items)
  mockAttachmentData.set(items[0].id, createMockImageData())
  mockAttachmentData.set(items[1].id, createMockPdfData())
  persistMockAttachments()
}

export const screwAttachmentApi = isTauri()
  ? {
      list: (screwSpecId: string) => invoke<ScrewAttachment[]>('list_screw_attachments', { screwSpecId }),
      counts: () => invoke<Record<string, number>>('get_screw_attachment_counts'),
      import: (screwSpecId: string, sourcePath: string) => invoke<ScrewAttachment>('import_screw_attachment', { screwSpecId, sourcePath }),
      importFile: async (_screwSpecId: string, _file: File): Promise<ScrewAttachment> => {
        throw new Error('桌面应用请通过系统文件选择器导入附件')
      },
      read: (attachmentId: string) => invoke<AttachmentContent>('read_screw_attachment', { attachmentId }),
      update: (attachmentId: string, data: { displayName?: string; annotations?: AttachmentAnnotation[]; sortOrder?: number }) =>
        invoke<ScrewAttachment>('update_screw_attachment', { attachmentId, ...data }),
      remove: (attachmentId: string) => invoke<boolean>('delete_screw_attachment', { attachmentId }),
    }
  : {
      list: async (screwSpecId: string) => {
        initializeMockAttachments()
        return (mockAttachments.get(screwSpecId) || []).map(item => ({ ...item, annotations: item.annotations.map(annotation => ({ ...annotation })) }))
      },
      counts: async () => {
        initializeMockAttachments()
        return Object.fromEntries([...mockAttachments.entries()].map(([id, items]) => [id, items.length]))
      },
      import: async (_screwSpecId: string, _sourcePath: string): Promise<ScrewAttachment> => {
        throw new Error('浏览器模式不支持文件路径导入')
      },
      importFile: async (screwSpecId: string, file: File): Promise<ScrewAttachment> => {
        initializeMockAttachments()
        if (!/^(image\/(png|jpeg|webp|gif)|application\/pdf)$/.test(file.type)) {
          throw new Error('仅支持 PNG、JPG、WEBP、GIF 和 PDF 文件')
        }
        if (file.size > 50 * 1024 * 1024) throw new Error('单个附件不能超过 50MB')
        const dataUrl = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader()
          reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
          reader.onerror = () => reject(new Error('读取附件失败'))
          reader.readAsDataURL(file)
        })
        const now = new Date().toISOString()
        const items = mockAttachments.get(screwSpecId) || []
        const attachment: ScrewAttachment = {
          id: crypto.randomUUID(), screwSpecId, displayName: file.name, fileName: file.name,
          mimeType: file.type, size: file.size, relativePath: `mock/${file.name}`,
          annotations: [], sortOrder: items.length, createdAt: now, updatedAt: now,
        }
        items.push(attachment)
        mockAttachments.set(screwSpecId, items)
        mockAttachmentData.set(attachment.id, dataUrl)
        persistMockAttachments()
        return { ...attachment }
      },
      read: async (attachmentId: string): Promise<AttachmentContent> => {
        for (const items of mockAttachments.values()) {
          const attachment = items.find(item => item.id === attachmentId)
          if (attachment) return { attachment: { ...attachment }, data: mockAttachmentData.get(attachmentId) || '' }
        }
        throw new Error('附件不存在')
      },
      update: async (attachmentId: string, data: { displayName?: string; annotations?: AttachmentAnnotation[]; sortOrder?: number }) => {
        for (const items of mockAttachments.values()) {
          const item = items.find(current => current.id === attachmentId)
          if (item) {
            Object.assign(item, data, { updatedAt: new Date().toISOString() })
            persistMockAttachments()
            return { ...item }
          }
        }
        throw new Error('附件不存在')
      },
      remove: async (attachmentId: string) => {
        for (const items of mockAttachments.values()) {
          const index = items.findIndex(item => item.id === attachmentId)
          if (index >= 0) {
            items.splice(index, 1)
            mockAttachmentData.delete(attachmentId)
            persistMockAttachments()
            return true
          }
        }
        return false
      },
    }

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
