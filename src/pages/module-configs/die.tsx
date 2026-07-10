import { Grid } from 'lucide-react'
import { dieApi, dieOrderApi, dieUseApi, dieLinkApi, dieStockApi } from '@/api'
import type { ModuleConfig } from './types'
import { Badge } from '@/components/ui/badge'

const statusMap: Record<string, { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  '正常': { label: '正常', variant: 'default' },
  '低库存': { label: '低库存', variant: 'secondary' },
  '缺货': { label: '缺货', variant: 'destructive' },
}

export const dieConfig: ModuleConfig = {
  name: '牙板管理',
  icon: Grid,
  stockType: 'die',
  infoApi: dieApi,
  orderApi: dieOrderApi,
  useApi: dieUseApi,
  linkApi: dieLinkApi,
  stockApi: dieStockApi,
  formFields: [
    { key: 'name', label: '名称', type: 'text', required: true },
    { key: 'machineType', label: '机台类型', type: 'select', required: true, options: ['CNC-1', 'CNC-2', 'CNC-3', 'CNC-4'] },
    { key: 'wireDiameter', label: '线径', type: 'text', required: true },
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
      accessorKey: 'machineType',
      header: '机台类型',
      size: 120,
      enableColumnFilter: true,
    },
    {
      accessorKey: 'wireDiameter',
      header: '线径',
      size: 100,
      enableSorting: true,
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
      header: '牙板名称',
      size: 160,
    },
    {
      accessorKey: 'screwSpec',
      header: '关联螺丝规格',
      size: 160,
    },
  ],
}
