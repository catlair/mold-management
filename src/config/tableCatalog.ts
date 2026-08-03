export interface TableColumnDefinition {
  id: string
  label: string
  defaultVisible?: boolean
  defaultSortable?: boolean
  defaultFilterable?: boolean
  canSort?: boolean
  canFilter?: boolean
}

export interface TableDefinition {
  id: string
  label: string
  columns: TableColumnDefinition[]
}

export interface PageTableDefinition {
  id: string
  label: string
  route: string
  tables: TableDefinition[]
}

const column = (
  id: string,
  label: string,
  options: Partial<Omit<TableColumnDefinition, 'id' | 'label'>> = {},
): TableColumnDefinition => ({
  id,
  label,
  defaultVisible: true,
  defaultSortable: false,
  defaultFilterable: false,
  canSort: true,
  canFilter: true,
  ...options,
})

const operationColumn = column('操作', '操作', {
  defaultSortable: false,
  defaultFilterable: false,
  canSort: false,
  canFilter: false,
})

const entityColumn = (label: string) => column(label, label, {
  defaultSortable: true,
  canSort: false,
  canFilter: false,
})

const stockColumns = (nameLabel: string): TableColumnDefinition[] => [
  column('name', nameLabel, { defaultSortable: true }),
  column('currentStock', '当前库存', { defaultSortable: true }),
  column('safetyStock', '安全库存', { defaultSortable: true }),
  column('status', '库存状态', { defaultSortable: true }),
]

const orderColumns = (entityLabel: string, withStatus = true): TableColumnDefinition[] => [
  entityColumn(entityLabel),
  column('quantity', '入库数量', { defaultSortable: true }),
  column('orderDate', '入库时间', { defaultSortable: true }),
  ...(withStatus ? [column('status', '到货状态', { defaultSortable: true, defaultFilterable: true })] : []),
  column('remark', '备注'),
]

const useColumns = (entityLabel: string, userLabel: string, quantityLabel: string, dateLabel: string): TableColumnDefinition[] => [
  entityColumn(entityLabel),
  column('user', userLabel, { defaultSortable: true }),
  column('quantity', quantityLabel, { defaultSortable: true }),
  column('useDate', dateLabel, { defaultSortable: true }),
  column('remark', '备注'),
]

const wireLinkColumns = (entityLabel: string): TableColumnDefinition[] => [
  entityColumn(entityLabel),
  column('wireMaterial', '线材规格', { defaultSortable: true }),
  column('remark', '备注'),
  operationColumn,
]

export const pageTableCatalog: PageTableDefinition[] = [
  {
    id: 'screw-spec',
    label: '螺丝规格',
    route: '/screw-spec',
    tables: [{
      id: 'screw-spec.info',
      label: '规格信息',
      columns: [
        column('name', '螺丝名称', { defaultSortable: true }),
        column('headType', '头型', { defaultSortable: true, defaultFilterable: true }),
        column('punch', '冲头', { defaultSortable: true }),
        column('threadType', '牙型', { defaultSortable: true, defaultFilterable: true }),
        column('die', '牙板', { defaultSortable: true }),
        column('headSize', '头/垫片大小', { defaultSortable: true }),
        column('headHeight', '头高', { defaultSortable: true }),
        column('length', '长度', { defaultSortable: true }),
        column('threadDiameter', '牙径', { defaultSortable: true }),
        column('shankLength', '光钉长度', { defaultSortable: true }),
        column('wireMaterial', '线材', { defaultSortable: true }),
        column('plating', '电镀', { defaultSortable: true, defaultFilterable: true }),
        column('customer', '客户名', { defaultSortable: true }),
        column('externalId', '外部ID', { defaultSortable: true }),
        column('remark', '备注'),
        column('附件', '附件', { canSort: false, canFilter: false }),
        operationColumn,
      ],
    }, {
      id: 'screw-spec.punch-dialog',
      label: '冲头关联弹窗',
      columns: [
        entityColumn('冲头名称'),
        column('spec', '规格'),
        column('material', '材质'),
        entityColumn('当前库存'),
        entityColumn('安全库存'),
        entityColumn('状态'),
        entityColumn('外显'),
      ],
    }, {
      id: 'screw-spec.die-dialog',
      label: '牙板关联弹窗',
      columns: [
        column('name', '牙板名称'),
        column('machineType', '机型'),
        column('wireDiameter', '线径'),
        entityColumn('当前库存'),
        entityColumn('安全库存'),
        entityColumn('状态'),
        entityColumn('外显'),
      ],
    }],
  },
  {
    id: 'punch',
    label: '冲头管理',
    route: '/punch',
    tables: [
      { id: 'punch.info', label: '冲头信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('spec', '规格', { defaultSortable: true }),
        column('material', '材质', { defaultSortable: true, defaultFilterable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'punch.order', label: '入库记录', columns: orderColumns('冲头') },
      { id: 'punch.use', label: '领用记录', columns: useColumns('冲头', '领用人', '领用数量', '领用时间') },
      { id: 'punch.link', label: '螺丝规格关联', columns: [entityColumn('冲头'), entityColumn('螺丝规格'), column('remark', '备注'), operationColumn] },
      { id: 'punch.linked-screws', label: '关联螺丝弹窗', columns: [
        column('name', '螺丝名称', { defaultSortable: true }), column('headType', '头型'),
        column('threadType', '牙型'), column('headSize', '头/垫片大小'), column('headHeight', '头高'),
        column('length', '长度'), column('threadDiameter', '牙径'), column('wireMaterial', '线材'), column('remark', '备注'),
      ] },
    ],
  },
  {
    id: 'die',
    label: '牙板管理',
    route: '/die',
    tables: [
      { id: 'die.info', label: '牙板信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('machineType', '机型', { defaultSortable: true, defaultFilterable: true }),
        column('wireDiameter', '线径', { defaultSortable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'die.order', label: '入库记录', columns: orderColumns('牙板名称', false) },
      { id: 'die.use', label: '领用记录', columns: useColumns('牙板名称', '领用人', '领用数量', '领用时间') },
      { id: 'die.link', label: '螺丝规格关联', columns: [entityColumn('牙板'), entityColumn('螺丝规格'), column('remark', '备注'), operationColumn] },
      { id: 'die.linked-screws', label: '关联螺丝弹窗', columns: [
        column('name', '螺丝名称', { defaultSortable: true }), column('headType', '头型'),
        column('threadType', '牙型'), column('headSize', '头/垫片大小'), column('headHeight', '头高'),
        column('length', '长度'), column('threadDiameter', '牙径'), column('wireMaterial', '线材'), column('remark', '备注'),
      ] },
    ],
  },
  {
    id: 'belt',
    label: '皮带管理',
    route: '/belt',
    tables: [
      { id: 'belt.info', label: '皮带信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('machine', '适用机器', { defaultSortable: true, defaultFilterable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'belt.order', label: '入库记录', columns: orderColumns('皮带') },
      { id: 'belt.use', label: '使用记录', columns: useColumns('皮带', '使用人', '使用数量', '使用时间') },
    ],
  },
  {
    id: 'main-mold',
    label: '主模具管理',
    route: '/main-mold',
    tables: [
      { id: 'main-mold.info', label: '主模具信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('holeDiameter', '孔径', { defaultSortable: true }),
        column('wireMaterial', '对应线材', { defaultSortable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'main-mold.order', label: '入库记录', columns: orderColumns('主模具') },
      { id: 'main-mold.use', label: '使用记录', columns: useColumns('主模具', '使用人', '使用数量', '使用时间') },
      { id: 'main-mold.link', label: '线材关联', columns: wireLinkColumns('主模具') },
    ],
  },
  {
    id: 'scissor',
    label: '剪刀管理',
    route: '/scissor',
    tables: [
      { id: 'scissor.info', label: '剪刀信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('diameter', '口径', { defaultSortable: true }),
        column('wireMaterial', '对应线材', { defaultSortable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'scissor.order', label: '入库记录', columns: orderColumns('剪刀') },
      { id: 'scissor.use', label: '使用记录', columns: useColumns('剪刀', '使用人', '使用数量', '使用时间') },
      { id: 'scissor.link', label: '线材关联', columns: wireLinkColumns('剪刀') },
    ],
  },
  {
    id: 'upper-punch',
    label: '上冲管理',
    route: '/upper-punch',
    tables: [
      { id: 'upper-punch.info', label: '上冲信息', columns: [
        column('name', '名称', { defaultSortable: true }),
        column('diameter', '口径', { defaultSortable: true }),
        column('wireMaterial', '对应线材', { defaultSortable: true }),
        column('safetyStock', '安全库存', { defaultSortable: true }),
        column('currentStock', '当前库存', { defaultSortable: true }),
        column('status', '库存状态', { defaultSortable: true }),
        column('remark', '备注'), operationColumn,
      ] },
      { id: 'upper-punch.order', label: '入库记录', columns: orderColumns('上冲') },
      { id: 'upper-punch.use', label: '使用记录', columns: useColumns('上冲', '使用人', '使用数量', '使用时间') },
      { id: 'upper-punch.link', label: '线材关联', columns: wireLinkColumns('上冲') },
    ],
  },
  {
    id: 'inventory',
    label: '库存汇总',
    route: '/inventory',
    tables: [
      { id: 'inventory.punch', label: '冲头库存', columns: stockColumns('冲头名称') },
      { id: 'inventory.die', label: '牙板库存', columns: stockColumns('牙板名称') },
      { id: 'inventory.belt', label: '皮带库存', columns: stockColumns('皮带名称') },
      { id: 'inventory.main-mold', label: '主模具库存', columns: stockColumns('主模具名称') },
      { id: 'inventory.scissor', label: '剪刀库存', columns: stockColumns('剪刀名称') },
      { id: 'inventory.upper-punch', label: '上冲库存', columns: stockColumns('上冲名称') },
    ],
  },
  {
    id: 'settings',
    label: '系统配置',
    route: '/settings',
    tables: [{
      id: 'settings.backups',
      label: '备份记录',
      columns: [
        column('backup_time', '备份时间'),
        column('backup_reason', '备份原因'),
        column('backup_md5', 'MD5'),
        entityColumn('锁定'),
        operationColumn,
      ],
    }],
  },
]

export const tableDefinitionMap = new Map(
  pageTableCatalog.flatMap(page => page.tables.map(table => [table.id, table] as const)),
)
