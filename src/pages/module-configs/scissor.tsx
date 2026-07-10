import { Scissors } from 'lucide-react'
import { scissorApi, scissorOrderApi, scissorUseApi, scissorLinkApi, scissorStockApi } from '@/api'
import type { ModuleConfig } from './types'
import { Badge } from '@/components/ui/badge'

const statusMap: Record<string, { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  '正常': { label: '正常', variant: 'default' },
  '低库存': { label: '低库存', variant: 'secondary' },
  '缺货': { label: '缺货', variant: 'destructive' },
}

export const scissorConfig: ModuleConfig = {
  name: '剪刀管理',
  icon: Scissors,
  stockType: 'scissor',
  infoApi: scissorApi,
  orderApi: scissorOrderApi,
  useApi: scissorUseApi,
  linkApi: scissorLinkApi,
  stockApi: scissorStockApi,
  formFields: [
    { key: 'name', label: '名称', type: 'text', required: true },
    { key: 'spec', label: '规格', type: 'text', required: true },
    { key: 'material', label: '材质', type: 'select', options: ['SKD11', 'SKH51', 'DC53', 'LD', 'ASP23'] },
    { key: 'safetyStock', label: '安全库存', type: 'number', defaultValue: 0 },
    { key: 'remark', label: '备注', type: 'text' },
  ],
  orderFormFields: [
    { key: 'date', label: '日期', type: 'text', required: true },
    { key: 'quantity', label: '数量', type: 'number', required: true },
    { key: 'supplier', label: '供应商', type: 'text' },
    { key: 'remark', label: '备注', type: 'text' },
  ],
  useFormFields: [
    { key: 'date', label: '日期', type: 'text', required: true },
    { key: 'quantity', label: '数量', type: 'number', required: true },
    { key: 'useType', label: '用途', type: 'text' },
    { key: 'remark', label: '备注', type: 'text' },
  ],
  infoColumns: [
    {
      accessorKey: 'name',
      header: '名称',
      size: 160,
      enableSorting: true,
    },
    {
      accessorKey: 'spec',
      header: '规格',
      size: 120,
      enableSorting: true,
    },
    {
      accessorKey: 'material',
      header: '材质',
      size: 120,
      enableColumnFilter: true,
    },
    {
      accessorKey: 'safetyStock',
      header: '安全库存',
      size: 100,
      enableSorting: true,
    },
    {
      accessorKey: 'currentStock',
      header: '当前库存',
      size: 100,
      enableSorting: true,
    },
    {
      accessorKey: 'status',
      header: '状态',
      size: 100,
      cell: ({ getValue }) => {
        const status = getValue() as string
        const config = statusMap[status] || { label: status, variant: 'outline' as const }
        return <Badge variant={config.variant}>{config.label}</Badge>
      },
    },
    {
      accessorKey: 'remark',
      header: '备注',
      size: 120,
    },
  ],
  orderColumns: [
    {
      accessorKey: 'date',
      header: '日期',
      size: 120,
      enableSorting: true,
    },
    {
      accessorKey: 'name',
      header: '名称',
      size: 160,
    },
    {
      accessorKey: 'spec',
      header: '规格',
      size: 120,
    },
    {
      accessorKey: 'quantity',
      header: '数量',
      size: 100,
      enableSorting: true,
    },
    {
      accessorKey: 'supplier',
      header: '供应商',
      size: 120,
    },
    {
      accessorKey: 'remark',
      header: '备注',
      size: 120,
    },
  ],
  useColumns: [
    {
      accessorKey: 'date',
      header: '日期',
      size: 120,
      enableSorting: true,
    },
    {
      accessorKey: 'name',
      header: '名称',
      size: 160,
    },
    {
      accessorKey: 'spec',
      header: '规格',
      size: 120,
    },
    {
      accessorKey: 'quantity',
      header: '数量',
      size: 100,
      enableSorting: true,
    },
    {
      accessorKey: 'useType',
      header: '用途',
      size: 120,
    },
    {
      accessorKey: 'remark',
      header: '备注',
      size: 120,
    },
  ],
  linkColumns: [
    {
      accessorKey: 'name',
      header: '剪刀名称',
      size: 160,
    },
    {
      accessorKey: 'wireSpec',
      header: '关联线材规格',
      size: 160,
    },
  ],
}
