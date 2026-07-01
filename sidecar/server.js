import { Hono } from 'hono'
import { serve } from '@hono/node-server'
import { cors } from 'hono/cors'
import ExcelJS from 'exceljs'
import { resolve } from 'path'
import { existsSync, mkdirSync } from 'fs'
import { readFile, writeFile, copyFile } from 'fs/promises'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// 打包后用可执行文件所在目录，开发时用脚本所在目录
const isPackaged = typeof process.pkg !== 'undefined'
const BASE_DIR = isPackaged ? dirname(process.execPath) : __dirname
const DATA_DIR = resolve(BASE_DIR, '..', 'data')
const FILE_PATH = resolve(DATA_DIR, 'mold-data.xlsx')

// 表名常量
const SHEETS = {
  SCREW_SPEC: '螺丝规格表',
  PUNCH_INFO: '冲头信息表',
  PUNCH_ORDER: '冲头订购记录',
  PUNCH_USE: '冲头领用记录',
  PUNCH_LINK: '冲头-螺丝规格关联',
  PUNCH_STOCK: '冲头库存汇总',
  DIE_INFO: '牙板信息表',
  DIE_ORDER: '牙板订购记录',
  DIE_USE: '牙板领用记录',
  DIE_LINK: '牙板-螺丝规格关联',
  DIE_STOCK: '牙板库存汇总',
  BELT_INFO: '皮带信息表',
  BELT_ORDER: '皮带订购记录',
  BELT_USE: '皮带使用记录',
  BELT_STOCK: '皮带库存汇总',
  MAIN_MOLD_INFO: '主模具信息表',
  MAIN_MOLD_ORDER: '主模具订购记录',
  MAIN_MOLD_USE: '主模具使用记录',
  MAIN_MOLD_LINK: '主模具-线材关联',
  MAIN_MOLD_STOCK: '主模具库存汇总',
  SCISSOR_INFO: '剪刀信息表',
  SCISSOR_ORDER: '剪刀订购记录',
  SCISSOR_USE: '剪刀使用记录',
  SCISSOR_LINK: '剪刀-线材关联',
  SCISSOR_STOCK: '剪刀库存汇总',
  UPPER_PUNCH_INFO: '上冲信息表',
  UPPER_PUNCH_ORDER: '上冲订购记录',
  UPPER_PUNCH_USE: '上冲使用记录',
  UPPER_PUNCH_LINK: '上冲-线材关联',
  UPPER_PUNCH_STOCK: '上冲库存汇总'
}

// 列定义
const COLUMN_DEFS = {
  [SHEETS.SCREW_SPEC]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '客户名', key: 'customer', width: 15 },
    { header: '外部ID', key: 'externalId', width: 15 },
    { header: '螺丝名称', key: 'name', width: 20 },
    { header: '螺丝头型', key: 'headType', width: 12 },
    { header: '冲头', key: 'punch', width: 12 },
    { header: '牙型', key: 'threadType', width: 12 },
    { header: '牙板', key: 'die', width: 12 },
    { header: '头/垫片大小', key: 'headSize', width: 12 },
    { header: '头高', key: 'headHeight', width: 10 },
    { header: '长度', key: 'length', width: 10 },
    { header: '牙径', key: 'threadDiameter', width: 10 },
    { header: '光钉长度', key: 'shankLength', width: 10 },
    { header: '线材', key: 'wireMaterial', width: 12 },
    { header: '电镀', key: 'plating', width: 12 },
    { header: '其他备注', key: 'remark', width: 20 }
  ],
  [SHEETS.PUNCH_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '规格', key: 'spec', width: 12 },
    { header: '材质', key: 'material', width: 12 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.PUNCH_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '冲头ID', key: 'punchId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.PUNCH_USE]: [
    { header: '领用ID', key: 'id', width: 15 },
    { header: '冲头ID', key: 'punchId', width: 15 },
    { header: '领用人', key: 'user', width: 12 },
    { header: '领用数量', key: 'quantity', width: 10 },
    { header: '领用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.PUNCH_LINK]: [
    { header: '关联ID', key: 'id', width: 15 },
    { header: '冲头ID', key: 'punchId', width: 15 },
    { header: '螺丝规格ID', key: 'screwSpecId', width: 15 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.PUNCH_STOCK]: [
    { header: '冲头ID', key: 'punchId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ],
  [SHEETS.DIE_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '机型', key: 'machineType', width: 12 },
    { header: '线径', key: 'wireDiameter', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.DIE_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '牙板ID', key: 'dieId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.DIE_USE]: [
    { header: '领用ID', key: 'id', width: 15 },
    { header: '牙板ID', key: 'dieId', width: 15 },
    { header: '领用人', key: 'user', width: 12 },
    { header: '领用数量', key: 'quantity', width: 10 },
    { header: '领用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.DIE_LINK]: [
    { header: '关联ID', key: 'id', width: 15 },
    { header: '牙板ID', key: 'dieId', width: 15 },
    { header: '螺丝规格ID', key: 'screwSpecId', width: 15 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.DIE_STOCK]: [
    { header: '牙板ID', key: 'dieId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ],
  [SHEETS.BELT_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '规格', key: 'spec', width: 12 },
    { header: '适用机器', key: 'machine', width: 12 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.BELT_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '皮带ID', key: 'beltId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.BELT_USE]: [
    { header: '使用ID', key: 'id', width: 15 },
    { header: '皮带ID', key: 'beltId', width: 15 },
    { header: '使用人', key: 'user', width: 12 },
    { header: '使用数量', key: 'quantity', width: 10 },
    { header: '使用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.BELT_STOCK]: [
    { header: '皮带ID', key: 'beltId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ],
  [SHEETS.MAIN_MOLD_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '孔径', key: 'holeDiameter', width: 10 },
    { header: '对应线材', key: 'wireMaterial', width: 12 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.MAIN_MOLD_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '主模具ID', key: 'mainMoldId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.MAIN_MOLD_USE]: [
    { header: '使用ID', key: 'id', width: 15 },
    { header: '主模具ID', key: 'mainMoldId', width: 15 },
    { header: '使用人', key: 'user', width: 12 },
    { header: '使用数量', key: 'quantity', width: 10 },
    { header: '使用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.MAIN_MOLD_LINK]: [
    { header: '关联ID', key: 'id', width: 15 },
    { header: '主模具ID', key: 'mainMoldId', width: 15 },
    { header: '线材规格', key: 'wireMaterial', width: 15 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.MAIN_MOLD_STOCK]: [
    { header: '主模具ID', key: 'mainMoldId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ],
  [SHEETS.SCISSOR_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '口径', key: 'diameter', width: 10 },
    { header: '对应线材', key: 'wireMaterial', width: 12 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.SCISSOR_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '剪刀ID', key: 'scissorId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.SCISSOR_USE]: [
    { header: '使用ID', key: 'id', width: 15 },
    { header: '剪刀ID', key: 'scissorId', width: 15 },
    { header: '使用人', key: 'user', width: 12 },
    { header: '使用数量', key: 'quantity', width: 10 },
    { header: '使用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.SCISSOR_LINK]: [
    { header: '关联ID', key: 'id', width: 15 },
    { header: '剪刀ID', key: 'scissorId', width: 15 },
    { header: '线材规格', key: 'wireMaterial', width: 15 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.SCISSOR_STOCK]: [
    { header: '剪刀ID', key: 'scissorId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ],
  [SHEETS.UPPER_PUNCH_INFO]: [
    { header: '内部ID', key: 'id', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '口径', key: 'diameter', width: 10 },
    { header: '对应线材', key: 'wireMaterial', width: 12 },
    { header: '安全库存', key: 'safetyStock', width: 10 }
  ],
  [SHEETS.UPPER_PUNCH_ORDER]: [
    { header: '订购ID', key: 'id', width: 15 },
    { header: '上冲ID', key: 'upperPunchId', width: 15 },
    { header: '订购数量', key: 'quantity', width: 10 },
    { header: '订购时间', key: 'orderDate', width: 15 },
    { header: '到货状态', key: 'status', width: 10 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.UPPER_PUNCH_USE]: [
    { header: '使用ID', key: 'id', width: 15 },
    { header: '上冲ID', key: 'upperPunchId', width: 15 },
    { header: '使用人', key: 'user', width: 12 },
    { header: '使用数量', key: 'quantity', width: 10 },
    { header: '使用时间', key: 'useDate', width: 22 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.UPPER_PUNCH_LINK]: [
    { header: '关联ID', key: 'id', width: 15 },
    { header: '上冲ID', key: 'upperPunchId', width: 15 },
    { header: '线材规格', key: 'wireMaterial', width: 15 },
    { header: '备注', key: 'remark', width: 20 }
  ],
  [SHEETS.UPPER_PUNCH_STOCK]: [
    { header: '上冲ID', key: 'upperPunchId', width: 15 },
    { header: '名称', key: 'name', width: 15 },
    { header: '当前库存', key: 'currentStock', width: 10 },
    { header: '安全库存', key: 'safetyStock', width: 10 },
    { header: '库存状态', key: 'status', width: 10 }
  ]
}

function ensureDataDir() {
  if (!existsSync(DATA_DIR)) {
    mkdirSync(DATA_DIR, { recursive: true })
  }
}

async function getWorkbook() {
  ensureDataDir()
  const workbook = new ExcelJS.Workbook()

  if (existsSync(FILE_PATH)) {
    await workbook.xlsx.readFile(FILE_PATH)
  } else {
    for (const [sheetName, columns] of Object.entries(COLUMN_DEFS)) {
      const sheet = workbook.addWorksheet(sheetName)
      sheet.columns = columns
    }
    await workbook.xlsx.writeFile(FILE_PATH)
  }

  return workbook
}

async function saveWorkbook(workbook) {
  await workbook.xlsx.writeFile(FILE_PATH)
}

async function getAll(sheetName) {
  const workbook = await getWorkbook()
  const sheet = workbook.getWorksheet(sheetName)
  if (!sheet) return []

  const items = []
  const columns = COLUMN_DEFS[sheetName]
  if (!columns) return []

  sheet.eachRow((row, rowNumber) => {
    if (rowNumber === 1) return
    const item = {}
    columns.forEach((col, index) => {
      item[col.key] = row.getCell(index + 1).value
    })
    items.push(item)
  })

  return items
}

async function getById(sheetName, id) {
  const items = await getAll(sheetName)
  return items.find(item => String(item.id) === String(id)) || null
}

function generateId(prefix) {
  const timestamp = Date.now().toString(36)
  const random = Math.random().toString(36).substring(2, 7)
  return `${prefix}_${timestamp}${random}`
}

async function add(sheetName, item) {
  const workbook = await getWorkbook()
  const sheet = workbook.getWorksheet(sheetName)
  if (!sheet) throw new Error(`工作表 ${sheetName} 不存在`)

  const columns = COLUMN_DEFS[sheetName]
  if (!columns) throw new Error(`未找到 ${sheetName} 的列定义`)

  // 自动生成ID
  if (!item.id) {
    const prefix = sheetName.replace(/表|记录|关联|汇总/g, '').substring(0, 2)
    item.id = generateId(prefix)
  }

  const rowData = columns.map(col => item[col.key] ?? '')
  sheet.addRow(rowData)

  await saveWorkbook(workbook)
  return item
}

async function update(sheetName, id, data) {
  const workbook = await getWorkbook()
  const sheet = workbook.getWorksheet(sheetName)
  if (!sheet) return null

  const columns = COLUMN_DEFS[sheetName]
  if (!columns) return null

  let found = false
  sheet.eachRow((row, rowNumber) => {
    if (rowNumber === 1) return
    if (String(row.getCell(1).value) === String(id)) {
      columns.forEach((col, index) => {
        if (data.hasOwnProperty(col.key)) {
          row.getCell(index + 1).value = data[col.key]
        }
      })
      found = true
    }
  })

  if (found) {
    await saveWorkbook(workbook)
    return getById(sheetName, id)
  }
  return null
}

async function remove(sheetName, id) {
  const workbook = await getWorkbook()
  const sheet = workbook.getWorksheet(sheetName)
  if (!sheet) return false

  let rowToDelete = null
  sheet.eachRow((row, rowNumber) => {
    if (rowNumber === 1) return
    if (String(row.getCell(1).value) === String(id)) {
      rowToDelete = rowNumber
    }
  })

  if (rowToDelete) {
    sheet.spliceRows(rowToDelete, 1)
    await saveWorkbook(workbook)
    return true
  }
  return false
}

// 库存计算配置
const STOCK_CONFIG = [
  { type: 'punch', info: SHEETS.PUNCH_INFO, order: SHEETS.PUNCH_ORDER, use: SHEETS.PUNCH_USE, stock: SHEETS.PUNCH_STOCK, itemIdKey: 'punchId' },
  { type: 'die', info: SHEETS.DIE_INFO, order: SHEETS.DIE_ORDER, use: SHEETS.DIE_USE, stock: SHEETS.DIE_STOCK, itemIdKey: 'dieId' },
  { type: 'belt', info: SHEETS.BELT_INFO, order: SHEETS.BELT_ORDER, use: SHEETS.BELT_USE, stock: SHEETS.BELT_STOCK, itemIdKey: 'beltId' },
  { type: 'mainMold', info: SHEETS.MAIN_MOLD_INFO, order: SHEETS.MAIN_MOLD_ORDER, use: SHEETS.MAIN_MOLD_USE, stock: SHEETS.MAIN_MOLD_STOCK, itemIdKey: 'mainMoldId' },
  { type: 'scissor', info: SHEETS.SCISSOR_INFO, order: SHEETS.SCISSOR_ORDER, use: SHEETS.SCISSOR_USE, stock: SHEETS.SCISSOR_STOCK, itemIdKey: 'scissorId' },
  { type: 'upperPunch', info: SHEETS.UPPER_PUNCH_INFO, order: SHEETS.UPPER_PUNCH_ORDER, use: SHEETS.UPPER_PUNCH_USE, stock: SHEETS.UPPER_PUNCH_STOCK, itemIdKey: 'upperPunchId' },
]

async function calculateStock(config) {
  const infoItems = await getAll(config.info)
  const orders = await getAll(config.order)
  const uses = await getAll(config.use)

  const stockData = infoItems.map(item => {
    const itemOrders = orders.filter(o => o[config.itemIdKey] === item.id && o.status === '已到货')
    const itemUses = uses.filter(u => u[config.itemIdKey] === item.id)
    const totalOrdered = itemOrders.reduce((sum, o) => sum + (Number(o.quantity) || 0), 0)
    const totalUsed = itemUses.reduce((sum, u) => sum + (Number(u.quantity) || 0), 0)
    const currentStock = totalOrdered - totalUsed
    const safetyStock = Number(item.safetyStock) || 0
    return {
      [config.itemIdKey]: item.id,
      name: item.name,
      currentStock,
      safetyStock,
      status: currentStock < safetyStock ? '需订购' : '安全'
    }
  })

  // 写回 Excel
  const workbook = await getWorkbook()
  const sheet = workbook.getWorksheet(config.stock)
  if (sheet) {
    // 清除旧数据（保留表头）
    while (sheet.rowCount > 1) {
      sheet.spliceRows(2, 1)
    }
    // 写入新数据
    const columns = COLUMN_DEFS[config.stock]
    stockData.forEach(item => {
      const rowData = columns.map(col => item[col.key] ?? '')
      sheet.addRow(rowData)
    })
    await saveWorkbook(workbook)
  }

  return stockData
}

// Hono HTTP 服务
const app = new Hono()

app.use('*', cors())

// 导出数据 — 返回 Excel 文件的 base64 编码
app.get('/api/export', async (c) => {
  try {
    const fileBuffer = await readFile(FILE_PATH)
    const base64 = fileBuffer.toString('base64')
    return c.json({ filename: 'mold-data.xlsx', data: base64 })
  } catch (error) {
    return c.json({ error: '导出失败: ' + error.message }, 500)
  }
})

// 导入数据 — 接收 base64 编码的 Excel 文件并替换当前数据
app.post('/api/import', async (c) => {
  try {
    const { data } = await c.req.json()
    if (!data) return c.json({ error: '未提供数据' }, 400)

    const buffer = Buffer.from(data, 'base64')

    // 先备份当前文件
    const backupPath = FILE_PATH.replace('.xlsx', `-backup-${Date.now()}.xlsx`)
    if (existsSync(FILE_PATH)) {
      await copyFile(FILE_PATH, backupPath)
    }

    // 写入新数据
    await writeFile(FILE_PATH, buffer)

    // 读取并统计各表数据量
    const workbook = new ExcelJS.Workbook()
    await workbook.xlsx.readFile(FILE_PATH)
    const stats = {}
    workbook.eachSheet((sheet) => {
      stats[sheet.name] = sheet.rowCount > 0 ? sheet.rowCount - 1 : 0
    })

    return c.json({ success: true, stats, backup: backupPath })
  } catch (error) {
    return c.json({ error: '导入失败: ' + error.message }, 500)
  }
})

// 生成ID
app.get('/api/generate-id/:prefix', (c) => {
  const prefix = c.req.param('prefix')
  return c.json({ id: generateId(prefix) })
})

// 计算单个类型库存
app.get('/api/calculate-stock/:type', async (c) => {
  try {
    const type = c.req.param('type')
    const config = STOCK_CONFIG.find(cfg => cfg.type === type)
    if (!config) return c.json({ error: '未知类型' }, 400)
    const result = await calculateStock(config)
    return c.json(result)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 计算所有库存
app.get('/api/calculate-stock', async (c) => {
  try {
    const results = {}
    for (const config of STOCK_CONFIG) {
      results[config.type] = await calculateStock(config)
    }
    return c.json(results)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 获取所有记录
app.get('/api/:sheet', async (c) => {
  try {
    const sheetName = decodeURIComponent(c.req.param('sheet'))
    const result = await getAll(sheetName)
    return c.json(result)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 获取单条记录
app.get('/api/:sheet/:id', async (c) => {
  try {
    const sheetName = decodeURIComponent(c.req.param('sheet'))
    const id = c.req.param('id')
    const result = await getById(sheetName, id)
    if (result) {
      return c.json(result)
    }
    return c.json({ error: '未找到' }, 404)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 添加记录
app.post('/api/:sheet', async (c) => {
  try {
    const sheetName = decodeURIComponent(c.req.param('sheet'))
    const data = await c.req.json()
    const result = await add(sheetName, data)
    return c.json(result, 201)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 更新记录
app.put('/api/:sheet/:id', async (c) => {
  try {
    const sheetName = decodeURIComponent(c.req.param('sheet'))
    const id = c.req.param('id')
    const data = await c.req.json()
    const result = await update(sheetName, id, data)
    if (result) {
      return c.json(result)
    }
    return c.json({ error: '未找到' }, 404)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

// 删除记录
app.delete('/api/:sheet/:id', async (c) => {
  try {
    const sheetName = decodeURIComponent(c.req.param('sheet'))
    const id = c.req.param('id')
    const result = await remove(sheetName, id)
    if (result) {
      return c.json({ success: true })
    }
    return c.json({ error: '未找到' }, 404)
  } catch (error) {
    return c.json({ error: error.message }, 500)
  }
})

const PORT = 3001

serve({
  fetch: app.fetch,
  port: PORT
}, (info) => {
  console.log(`Excel 服务已启动: http://localhost:${info.port}`)
})
