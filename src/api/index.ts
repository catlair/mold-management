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
      const name = created.name || created.id
      writeMockLog('add', sheetName, created.id, `新增 ${sheetName}：${name}`)
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
      const changes = Object.entries(data)
        .filter(([key, value]) => key !== 'id' && String(current[key] ?? '') !== String(value ?? ''))
        .map(([key, value]) => `${key}: ${String(current[key] ?? '') || '空'}→${String(value ?? '') || '空'}`)
      const name = updated.name || id
      writeMockLog('update', sheetName, id, changes.length
        ? `修改 ${sheetName}：${name}（${changes.join('；')}）`
        : `修改 ${sheetName}：${name}`)
      return { ...updated }
    },
    remove: async (id: string) => {
      const currentIndex = rows.findIndex(row => row.id === id)
      if (currentIndex < 0) return false
      const removed = rows[currentIndex]
      rows.splice(currentIndex, 1)
      writeMockLog('delete', sheetName, id, `删除 ${sheetName}：${removed.name || id}`)
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
// 导出分组：螺丝规格 / 冲头 / 牙板 / 皮带 / 主模具 / 剪刀 / 上冲
export const EXPORT_GROUPS = [
  { id: '螺丝规格', label: '螺丝规格' },
  { id: '冲头', label: '冲头' },
  { id: '牙板', label: '牙板' },
  { id: '皮带', label: '皮带' },
  { id: '主模具', label: '主模具' },
  { id: '剪刀', label: '剪刀' },
  { id: '上冲', label: '上冲' }
] as const

export interface ExcelSheetInfo {
  name: string
  table: string
  matchedByHeader: boolean
  rowCount: number
  systemCalculated: boolean
}

export interface ExcelSheetSelection {
  name: string
  table: string
}

export const dataApi = isTauri()
  ? {
      exportGroup: (groupId: string, destinationPath: string) =>
        invoke<{ success: boolean; filePath: string; group: string }>('export_excel_group', { groupId, destinationPath }),
      listExcelSheets: (sourcePath: string) =>
        invoke<ExcelSheetInfo[]>('list_excel_sheets', { sourcePath }),
      importExcelSheets: (sourcePath: string, selections: ExcelSheetSelection[]) =>
        invoke<{ success: boolean; stats: Record<string, number> }>('import_excel_sheets', { sourcePath, selections }),
      exportPackage: (destinationPath: string) => invoke<{ success: boolean; filePath: string }>('export_data_package', { destinationPath }),
      importPackage: (sourcePath: string) => invoke<{ success: boolean; stats: Record<string, number>; attachmentCount: number }>('import_data_package', { sourcePath }),
    }
  : {
      exportGroup: async (groupId: string, destinationPath: string) => ({ success: true, filePath: destinationPath, group: groupId }),
      listExcelSheets: async () => ([
        { name: '螺丝规格表', table: '螺丝规格表', matchedByHeader: false, rowCount: 40, systemCalculated: false },
        { name: '冲头信息表', table: '冲头信息表', matchedByHeader: false, rowCount: 21, systemCalculated: false },
        { name: '冲头库存汇总', table: '冲头库存汇总', matchedByHeader: false, rowCount: 21, systemCalculated: true }
      ] as ExcelSheetInfo[]),
      importExcelSheets: async (_sourcePath: string, _selections: ExcelSheetSelection[]) => ({ success: true, stats: {} }),
      exportPackage: async (destinationPath: string) => ({ success: true, filePath: destinationPath }),
      importPackage: async (_sourcePath: string) => ({ success: true, stats: {}, attachmentCount: 0 }),
    }

export interface WebDavConfigView {
  url: string
  remotePath: string
  usernameMasked: string
  credentialsConfigured: boolean
  credentialStore: string
  usingDevelopmentConfig: boolean
  autoUploadOnStart: boolean
  autoUploadOnExit: boolean
  lastEtag: string | null
  lastUploadedAt: string | null
  lastDownloadedAt: string | null
}

export interface WebDavRemoteStatus {
  connected: boolean
  exists: boolean
  remotePath: string
  etag: string | null
  lastModified: string | null
  size: number | null
}

export interface WebDavConfigInput {
  url: string
  remotePath: string
  username?: string
  password?: string
  autoUploadOnStart: boolean
  autoUploadOnExit: boolean
}

export const webdavApi = isTauri()
  ? {
      getConfig: () => invoke<WebDavConfigView>('get_webdav_config'),
      setConfig: (input: WebDavConfigInput) => invoke<{ success: boolean }>('set_webdav_config', { ...input }),
      testConnection: () => invoke<WebDavRemoteStatus>('test_webdav_connection'),
      getStatus: () => invoke<WebDavRemoteStatus>('get_webdav_status'),
      upload: (forceOverwrite = false) => invoke<{ success: boolean; etag: string | null; uploadedAt: string; size: number; sha256: string }>('upload_webdav_now', { forceOverwrite }),
      download: () => invoke<{ success: boolean; etag: string | null; sha256: string; checksumVerified: boolean }>('download_webdav_now'),
    }
  : {
      getConfig: async (): Promise<WebDavConfigView> => ({
        url: 'https://dav.example.com/dav/', remotePath: 'mold-management.moldpkg',
        usernameMasked: 'd***@example.com', credentialsConfigured: true, credentialStore: '浏览器预览模式',
        usingDevelopmentConfig: true, autoUploadOnStart: false, autoUploadOnExit: false,
        lastEtag: null, lastUploadedAt: null, lastDownloadedAt: null,
      }),
      setConfig: async () => ({ success: true }),
      testConnection: async (): Promise<WebDavRemoteStatus> => ({
        connected: true, exists: false, remotePath: 'mold-management.moldpkg', etag: null,
        lastModified: null, size: null,
      }),
      getStatus: async (): Promise<WebDavRemoteStatus> => ({
        connected: true, exists: false, remotePath: 'mold-management.moldpkg', etag: null,
        lastModified: null, size: null,
      }),
      upload: async () => ({ success: true, etag: 'mock-etag', uploadedAt: new Date().toISOString(), size: 0, sha256: '' }),
      download: async () => ({ success: true, etag: 'mock-etag', sha256: '', checksumVerified: true }),
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

export const dieMachineTypeApi = isTauri()
  ? {
      get: () => invoke<string[]>('get_die_machine_types'),
      set: (machineTypes: string[]) => invoke<{ success: boolean; machineTypes: string[] }>('set_die_machine_types', { machineTypes }),
    }
  : {
      get: async () => ['003', '3/16', '1/4', '6R'],
      set: async (machineTypes: string[]) => ({ success: true, machineTypes }),
    }

export const punchSpecApi = isTauri()
  ? {
      get: () => invoke<string[]>('get_punch_specs'),
      set: (specs: string[]) => invoke<{ success: boolean; specs: string[] }>('set_punch_specs', { specs }),
    }
  : {
      get: async () => ['12*15', '14*15', '18*18'],
      set: async (specs: string[]) => ({ success: true, specs }),
    }

export type AgentProtocol = 'openai' | 'anthropic' | 'gemini'

export interface AgentProfileView {
  id: string
  name: string
  kind: 'builtin' | 'custom'
  provider: string
  format: string
  endpoint: string
  model: string
  apiKeyConfigured: boolean
  /** 该配置是否需要 API Key（false = 免费服务如 opencode Zen） */
  apiKeyRequired?: boolean
}

export interface AgentBuiltinView {
  value: string
  label: string
  model: string
  protocol: string
  /** 是否需要 API Key（false = 免费服务，无需填写 Key） */
  needsApiKey?: boolean
}

export interface ZenModelView {
  id: string
  label: string
  protocol: string
  free: boolean
}

export interface AgentConfigView {
  profiles: AgentProfileView[]
  active: string
  credentialStore: string
  builtins: AgentBuiltinView[]
  ccEndpoint?: string
  ccModels?: ZenModelView[]
  zenFreeModels?: ZenModelView[]
  zenModels?: ZenModelView[]
}

export interface AgentProfileInput {
  id: string
  name: string
  kind: 'builtin' | 'custom'
  provider: string
  format: string
  endpoint: string
  model: string
  apiKey?: string
}

export interface AgentChange {
  operation: 'add' | 'update' | 'delete' | 'set_setting' | 'import'
  table: string
  id?: string
  fields: Record<string, unknown>
  before?: Record<string, unknown> | null
  after?: Record<string, unknown> | null
  requireConfirm?: boolean
}

export interface AgentChatResult {
  answer: string
  changes: AgentChange[] | null
  reasoning?: string | null
}

export interface AgentExcelCompareTable {
  table: string
  matchKey: 'business' | 'id' | 'none'
  xlsxCount: number
  dbCount: number
  addedCount: number
  modifiedCount: number
  removedCount: number
  unchanged: number
  skipReason?: string
  added?: Record<string, string>[]
  modified?: { key: string; changes: Record<string, [string, string]> }[]
  removed?: string[]
}

export type AgentExcelCompareResult = AgentExcelCompareTable[]

export const agentApi = isTauri()
  ? {
      getConfig: () => invoke<AgentConfigView>('get_agent_config'),
      setConfig: (input: { profiles: AgentProfileInput[]; active: string }) =>
        invoke<{ success: boolean }>('set_agent_config', { ...input }),
      chat: (question: string, history: { role: string; content: string }[], pageContext = '') =>
        invoke<AgentChatResult>('agent_chat', { question, history, pageContext }),
      applyChange: (change: AgentChange) =>
        invoke<{ success: boolean; operation: string; table: string; result: unknown }>('apply_agent_change', { change }),
      applyChanges: (changes: AgentChange[]) =>
        invoke<{ success: boolean; applied: { success: boolean; operation: string; table: string; result: unknown }[]; failed: { change: AgentChange; error: string }[] }>('apply_agent_changes', { changes }),
      compareExcel: (xlsxPath: string) =>
        invoke<AgentExcelCompareResult>('compare_excel', { xlsxPath }),
      analyzeExcel: (xlsxPath: string, history: { role: string; content: string }[], pageContext = '') =>
        invoke<AgentChatResult>('agent_analyze_excel', { xlsxPath, history, pageContext }),
    }
  : {
      getConfig: async (): Promise<AgentConfigView> => ({
        profiles: [],
        active: '',
        credentialStore: '浏览器预览模式',
        builtins: [
          { value: 'deepseek', label: 'DeepSeek', model: 'deepseek-v4-flash', protocol: 'openai', needsApiKey: true },
          { value: 'openai', label: 'OpenAI', model: 'gpt-4.1-mini', protocol: 'openai', needsApiKey: true },
          { value: 'glm', label: '智谱 GLM', model: 'glm-4-flash-250414', protocol: 'openai', needsApiKey: true },
          { value: 'anthropic', label: 'Claude Code', model: 'claude-sonnet-4-6', protocol: 'anthropic', needsApiKey: true },
          { value: 'opencode-zen-free', label: 'OpenCode Zen（免费）', model: 'deepseek-v4-flash-free', protocol: 'openai', needsApiKey: false },
          { value: 'opencode-zen', label: 'OpenCode Zen（需 Key）', model: 'deepseek-v4-flash', protocol: 'openai', needsApiKey: true },
          { value: 'qwen', label: '通义千问', model: 'qwen-plus', protocol: 'openai', needsApiKey: true },
          { value: 'gemini', label: 'Gemini', model: 'gemini-3.6-flash', protocol: 'gemini', needsApiKey: true },
        ],
        ccModels: [
          { id: 'deepseek-v4-flash-free', label: 'DeepSeek V4 Flash Free', protocol: 'openai', free: true },
          { id: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash', protocol: 'openai', free: false },
        ],
        zenFreeModels: [
          { id: 'big-pickle', label: 'Big Pickle', protocol: 'openai', free: true },
          { id: 'deepseek-v4-flash-free', label: 'DeepSeek V4 Flash Free', protocol: 'openai', free: true },
        ],
        zenModels: [
          { id: 'gpt-5.6-sol', label: 'GPT 5.6 Sol', protocol: 'responses', free: false },
          { id: 'claude-sonnet-4-6', label: 'Claude Sonnet 4.6', protocol: 'anthropic', free: false },
          { id: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash', protocol: 'openai', free: false },
          { id: 'deepseek-v4-flash-free', label: 'DeepSeek V4 Flash Free', protocol: 'openai', free: true },
        ],
      }),
      setConfig: async () => ({ success: true }),
      chat: async (question: string): Promise<AgentChatResult> => ({
        answer: `浏览器预览模式已收到：${question}。桌面应用中会读取当前业务数据并调用已配置的第三方 API。`,
        changes: null,
      }),
      applyChange: async () => ({ success: true, operation: 'update', table: '', result: {} }),
      applyChanges: async () => ({ success: true, applied: [], failed: [] }),
      compareExcel: async () => [],
      analyzeExcel: async (): Promise<AgentChatResult> => ({ answer: '浏览器预览模式不支持文件分析', changes: null }),
    }

// ========== 操作日志 ==========
export interface OperationLogEntry {
  id: number
  ts: number
  tableName: string
  operation: 'add' | 'update' | 'delete' | 'import'
  recordId: string
  summary: string
}

// 浏览器预览模式：mock 日志持久化到 localStorage
const MOCK_LOG_STORAGE_KEY = 'mold-management-mock-operation-logs'

function readMockLogs(): OperationLogEntry[] {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(MOCK_LOG_STORAGE_KEY) : null
    return raw ? JSON.parse(raw) as OperationLogEntry[] : []
  } catch {
    return []
  }
}

function writeMockLog(operation: OperationLogEntry['operation'], tableName: string, recordId: string, summary: string) {
  try {
    const logs = readMockLogs()
    logs.unshift({ id: Date.now(), ts: Math.floor(Date.now() / 1000), tableName, operation, recordId, summary })
    if (logs.length > 500) logs.length = 500
    localStorage.setItem(MOCK_LOG_STORAGE_KEY, JSON.stringify(logs))
  } catch {
    // mock 环境写失败不影响业务
  }
}

export const operationLogApi = isTauri()
  ? {
      get: (limit = 500, offset = 0) =>
        invoke<{ total: number; items: OperationLogEntry[] }>('get_operation_logs', { limit, offset }),
      clear: () => invoke<boolean>('clear_operation_logs'),
    }
  : {
      get: async (limit = 500, _offset = 0) => {
        const items = readMockLogs().slice(0, limit)
        return { total: items.length, items }
      },
      clear: async () => {
        localStorage.removeItem(MOCK_LOG_STORAGE_KEY)
        return true
      },
    }
