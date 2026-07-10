import { useState, useEffect, useCallback } from 'react'
import {
  Plus,
  Maximize2,
  Minimize2,
  Pencil,
  Trash2,
  ExternalLink,
} from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { InputNumber } from '@/components/ui/input-number'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { DataTable } from '@/components/data-table'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { useFullscreen } from '@/hooks/use-fullscreen'
import { cn } from '@/lib/utils'
import { screwSpecApi } from '@/api'
import type { ModuleConfig, FormField } from './module-configs/types'

interface ModulePageProps {
  config: ModuleConfig
}

export default function ModulePage({ config }: ModulePageProps) {
  const { isFullscreen, toggle: toggleFullscreen } = useFullscreen()

  // Data states
  const [infoData, setInfoData] = useState<any[]>([])
  const [orderData, setOrderData] = useState<any[]>([])
  const [useData, setUseData] = useState<any[]>([])
  const [linkData, setLinkData] = useState<any[]>([])
  const [screwSpecs, setScrewSpecs] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  // Dialog states
  const [infoDialogOpen, setInfoDialogOpen] = useState(false)
  const [orderDialogOpen, setOrderDialogOpen] = useState(false)
  const [useDialogOpen, setUseDialogOpen] = useState(false)
  const [linkDialogOpen, setLinkDialogOpen] = useState(false)
  const [editingItem, setEditingItem] = useState<any>(null)
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false)
  const [deletingItem, setDeletingItem] = useState<any>(null)

  // Form states
  const [infoForm, setInfoForm] = useState<Record<string, any>>({})
  const [orderForm, setOrderForm] = useState<Record<string, any>>({})
  const [useForm, setUseForm] = useState<Record<string, any>>({})
  const [linkForm, setLinkForm] = useState<Record<string, any>>({})

  // Screw specs popup
  const [screwSpecsPopup, setScrewSpecsPopup] = useState<{ open: boolean; specs: string[] }>({
    open: false,
    specs: [],
  })

  // Load data on mount
  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const promises: Promise<any>[] = [
        config.infoApi.getAll(),
        config.orderApi.getAll(),
        config.useApi.getAll(),
        config.stockApi ? config.stockApi.getAll() : Promise.resolve([]),
      ]

      if (config.linkApi) {
        promises.push(config.linkApi.getAll())
      }

      const [info, orders, uses, stock, links] = await Promise.all(promises)

      // Merge stock data into info
      const stockMap = new Map<string, any>()
      if (Array.isArray(stock)) {
        stock.forEach((s: any) => stockMap.set(s.name, s))
      }

      const mergedInfo = Array.isArray(info)
        ? info.map((item: any) => {
            const stockInfo = stockMap.get(item.name)
            return {
              ...item,
              currentStock: stockInfo?.currentStock ?? 0,
              status: stockInfo?.status ?? '正常',
            }
          })
        : []

      setInfoData(mergedInfo)
      setOrderData(Array.isArray(orders) ? orders : [])
      setUseData(Array.isArray(uses) ? uses : [])
      if (links) {
        setLinkData(Array.isArray(links) ? links : [])
      }

      // Load screw specs for linked modules
      if (config.linkApi) {
        const specs = await screwSpecApi.getAll()
        setScrewSpecs(Array.isArray(specs) ? specs : [])
      }
    } catch (error) {
      console.error('Failed to load data:', error)
    } finally {
      setLoading(false)
    }
  }, [config])

  useEffect(() => {
    loadData()
  }, [loadData])

  // Get unique values for filter columns
  const getUniqueValues = (data: any[], key: string) => {
    const values = new Set(data.map((item) => item[key]).filter(Boolean))
    return Array.from(values)
  }

  // Handle form field change
  const handleFormChange = (
    formType: 'info' | 'order' | 'use' | 'link',
    key: string,
    value: any
  ) => {
    const setter =
      formType === 'info'
        ? setInfoForm
        : formType === 'order'
        ? setOrderForm
        : formType === 'use'
        ? setUseForm
        : setLinkForm
    setter((prev) => ({ ...prev, [key]: value }))
  }

  // Render form field
  const renderFormField = (
    field: FormField,
    formType: 'info' | 'order' | 'use' | 'link',
    form: Record<string, any>
  ) => {
    const value = form[field.key] ?? field.defaultValue ?? ''

    if (field.type === 'select' && field.options) {
      // Get unique values from data for filter dropdowns
      const data =
        formType === 'info'
          ? infoData
          : formType === 'order'
          ? orderData
          : formType === 'use'
          ? useData
          : linkData
      const options = field.options.length > 0 ? field.options : getUniqueValues(data, field.key)

      return (
        <Select
          key={field.key}
          value={value}
          onValueChange={(v) => handleFormChange(formType, field.key, v)}
        >
          <SelectTrigger>
            <SelectValue placeholder={`选择${field.label}`} />
          </SelectTrigger>
          <SelectContent>
            {options.map((opt) => (
              <SelectItem key={opt} value={opt}>
                {opt}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )
    }

    if (field.type === 'number') {
      return (
        <InputNumber
          key={field.key}
          value={value}
          onChange={(v) => handleFormChange(formType, field.key, v)}
          min={0}
        />
      )
    }

    return (
      <Input
        key={field.key}
        value={value}
        onChange={(e) => handleFormChange(formType, field.key, e.target.value)}
        placeholder={field.placeholder || `请输入${field.label}`}
      />
    )
  }

  // Add info columns with actions
  const infoColumnsWithActions = [
    ...config.infoColumns.map((col) => {
      // Make name column clickable for linked modules
      if ((col as any).accessorKey === 'name' && config.linkApi) {
        return {
          ...col,
          cell: ({ getValue }: any) => {
            const name = getValue() as string
            const linkedSpecs = linkData
              .filter((link) => link.name === name)
              .map((link) => link.screwSpec || link.wireSpec)
              .filter(Boolean)

            return (
              <button
                className="text-primary hover:underline cursor-pointer text-left font-medium"
                onClick={() => {
                  if (linkedSpecs.length > 0) {
                    setScrewSpecsPopup({ open: true, specs: linkedSpecs })
                  }
                }}
              >
                {name}
                {linkedSpecs.length > 0 && (
                  <ExternalLink className="inline ml-1 h-3 w-3" />
                )}
              </button>
            )
          },
        }
      }
      return col
    }),
    {
      id: 'actions',
      header: '操作',
      size: 120,
      cell: ({ row }: any) => (
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setEditingItem(row.original)
              setInfoForm(row.original)
              setInfoDialogOpen(true)
            }}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setDeletingItem(row.original)
              setDeleteConfirmOpen(true)
            }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
  ]

  // Order columns with actions
  const orderColumnsWithActions = [
    ...config.orderColumns,
    {
      id: 'actions',
      header: '操作',
      size: 80,
      cell: ({ row }: any) => (
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setEditingItem(row.original)
              setOrderForm(row.original)
              setOrderDialogOpen(true)
            }}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setDeletingItem({ ...row.original, type: 'order' })
              setDeleteConfirmOpen(true)
            }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
  ]

  // Use columns with actions
  const useColumnsWithActions = [
    ...config.useColumns,
    {
      id: 'actions',
      header: '操作',
      size: 80,
      cell: ({ row }: any) => (
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setEditingItem(row.original)
              setUseForm(row.original)
              setUseDialogOpen(true)
            }}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setDeletingItem({ ...row.original, type: 'use' })
              setDeleteConfirmOpen(true)
            }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
  ]

  // Link columns with actions
  const linkColumnsWithActions = config.linkApi
    ? [
        ...(config.linkColumns || []),
        {
          id: 'actions',
          header: '操作',
          size: 80,
          cell: ({ row }: any) => (
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="icon"
                onClick={() => {
                  setEditingItem(row.original)
                  setLinkForm(row.original)
                  setLinkDialogOpen(true)
                }}
              >
                <Pencil className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => {
                  setDeletingItem({ ...row.original, type: 'link' })
                  setDeleteConfirmOpen(true)
                }}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          ),
        },
      ]
    : []

  // Handle save
  const handleSave = async (type: 'info' | 'order' | 'use' | 'link') => {
    const forms = {
      info: { form: infoForm, api: config.infoApi, data: infoData, setData: setInfoData },
      order: { form: orderForm, api: config.orderApi, data: orderData, setData: setOrderData },
      use: { form: useForm, api: config.useApi, data: useData, setData: setUseData },
      link: { form: linkForm, api: config.linkApi!, data: linkData, setData: setLinkData },
    }

    const { form, api } = forms[type]

    // Validate required fields
    const fields =
      type === 'info'
        ? config.formFields
        : type === 'order'
        ? config.orderFormFields || []
        : type === 'use'
        ? config.useFormFields || []
        : []

    for (const field of fields) {
      if (field.required && !form[field.key]) {
        alert(`请填写${field.label}`)
        return
      }
    }

    try {
      if (editingItem) {
        await api.update(editingItem.id, form)
      } else {
        await api.add(form)
      }
      await loadData()
      closeDialog(type)
    } catch (error) {
      console.error('Save failed:', error)
    }
  }

  // Handle delete
  const handleDelete = async () => {
    if (!deletingItem) return

    try {
      let api = config.infoApi
      if (deletingItem.type === 'order') {
        api = config.orderApi
      } else if (deletingItem.type === 'use') {
        api = config.useApi
      } else if (deletingItem.type === 'link') {
        api = config.linkApi!
      }

      await api.remove(deletingItem.id)
      await loadData()
      setDeleteConfirmOpen(false)
      setDeletingItem(null)
    } catch (error) {
      console.error('Delete failed:', error)
    }
  }

  // Close dialog
  const closeDialog = (type: 'info' | 'order' | 'use' | 'link') => {
    if (type === 'info') {
      setInfoDialogOpen(false)
      setInfoForm({})
    } else if (type === 'order') {
      setOrderDialogOpen(false)
      setOrderForm({})
    } else if (type === 'use') {
      setUseDialogOpen(false)
      setUseForm({})
    } else {
      setLinkDialogOpen(false)
      setLinkForm({})
    }
    setEditingItem(null)
  }

  // Open add dialog
  const openAddDialog = (type: 'info' | 'order' | 'use' | 'link') => {
    const defaults: Record<string, any> = {}
    const fields =
      type === 'info'
        ? config.formFields
        : type === 'order'
        ? config.orderFormFields || []
        : type === 'use'
        ? config.useFormFields || []
        : config.linkColumns?.map((col) => ({
            key: (col as any).accessorKey as string,
            label: (col as any).header as string,
            type: 'text' as const,
          })) || []

    fields.forEach((f) => {
      defaults[f.key] = (f as any).defaultValue ?? ''
    })

    if (type === 'info') {
      setInfoForm(defaults)
      setInfoDialogOpen(true)
    } else if (type === 'order') {
      setOrderForm(defaults)
      setOrderDialogOpen(true)
    } else if (type === 'use') {
      setUseForm(defaults)
      setUseDialogOpen(true)
    } else {
      setLinkForm(defaults)
      setLinkDialogOpen(true)
    }
  }

  // Render dialog form fields
  const renderDialogFields = (
    fields: FormField[],
    formType: 'info' | 'order' | 'use' | 'link',
    form: Record<string, any>
  ) => {
    return fields.map((field) => (
      <div key={field.key} className="grid gap-2">
        <Label htmlFor={field.key}>
          {field.label}
          {field.required && <span className="text-destructive ml-1">*</span>}
        </Label>
        {renderFormField(field, formType, form)}
      </div>
    ))
  }

  return (
    <div
      className={cn(
        'flex flex-col',
        isFullscreen ? 'h-screen' : 'h-full'
      )}
    >
      <Card className={cn('flex flex-col', isFullscreen ? 'flex-1 rounded-none border-0' : '')}>
        <CardHeader className={cn('flex flex-row items-center justify-between', isFullscreen && 'py-2')}>
          <CardTitle className="flex items-center gap-2">
            <config.icon className="h-5 w-5" />
            {config.name}
          </CardTitle>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => openAddDialog('info')}
            >
              <Plus className="h-4 w-4 mr-1" />
              新增
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleFullscreen}
            >
              {isFullscreen ? (
                <Minimize2 className="h-4 w-4" />
              ) : (
                <Maximize2 className="h-4 w-4" />
              )}
            </Button>
          </div>
        </CardHeader>
        <CardContent className={cn('flex-1 overflow-hidden', isFullscreen && 'p-2')}>
          <Tabs defaultValue="info" className="h-full flex flex-col">
            <TabsList>
              <TabsTrigger value="info">信息</TabsTrigger>
              <TabsTrigger value="orders">入库记录</TabsTrigger>
              <TabsTrigger value="usage">领用记录</TabsTrigger>
              {config.linkApi && <TabsTrigger value="link">关联</TabsTrigger>}
            </TabsList>

            <TabsContent
              value="info"
              className={cn(
                'flex-1 overflow-auto',
                isFullscreen && 'mt-0'
              )}
            >
              <div className="h-full">
                <DataTable
                  columns={infoColumnsWithActions}
                  data={infoData}
                  loading={loading}
                  emptyText="暂无数据"
                />
              </div>
            </TabsContent>

            <TabsContent
              value="orders"
              className={cn(
                'flex-1 overflow-auto',
                isFullscreen && 'mt-0'
              )}
            >
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">入库记录</h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => openAddDialog('order')}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  新增
                </Button>
              </div>
              <div className="h-full">
                <DataTable
                  columns={orderColumnsWithActions}
                  data={orderData}
                  loading={loading}
                  emptyText="暂无入库记录"
                />
              </div>
            </TabsContent>

            <TabsContent
              value="usage"
              className={cn(
                'flex-1 overflow-auto',
                isFullscreen && 'mt-0'
              )}
            >
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">领用记录</h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => openAddDialog('use')}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  新增
                </Button>
              </div>
              <div className="h-full">
                <DataTable
                  columns={useColumnsWithActions}
                  data={useData}
                  loading={loading}
                  emptyText="暂无领用记录"
                />
              </div>
            </TabsContent>

            {config.linkApi && (
              <TabsContent
                value="link"
                className={cn(
                  'flex-1 overflow-auto',
                  isFullscreen && 'mt-0'
                )}
              >
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-semibold">关联记录</h3>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => openAddDialog('link')}
                  >
                    <Plus className="h-4 w-4 mr-1" />
                    新增
                  </Button>
                </div>
                <div className="h-full">
                  <DataTable
                    columns={linkColumnsWithActions}
                    data={linkData}
                    loading={loading}
                    emptyText="暂无关联记录"
                  />
                </div>
              </TabsContent>
            )}
          </Tabs>
        </CardContent>
      </Card>

      {/* Info Dialog */}
      <Dialog open={infoDialogOpen} onOpenChange={(open) => !open && closeDialog('info')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingItem ? '编辑' : '新增'}{config.name.replace('管理', '')}</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {renderDialogFields(config.formFields, 'info', infoForm)}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => closeDialog('info')}>
              取消
            </Button>
            <Button onClick={() => handleSave('info')}>
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Order Dialog */}
      <Dialog open={orderDialogOpen} onOpenChange={(open) => !open && closeDialog('order')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingItem ? '编辑' : '新增'}入库记录</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {renderDialogFields(config.orderFormFields || [], 'order', orderForm)}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => closeDialog('order')}>
              取消
            </Button>
            <Button onClick={() => handleSave('order')}>
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Use Dialog */}
      <Dialog open={useDialogOpen} onOpenChange={(open) => !open && closeDialog('use')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingItem ? '编辑' : '新增'}领用记录</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {renderDialogFields(config.useFormFields || [], 'use', useForm)}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => closeDialog('use')}>
              取消
            </Button>
            <Button onClick={() => handleSave('use')}>
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Link Dialog */}
      {config.linkApi && (
        <Dialog open={linkDialogOpen} onOpenChange={(open) => !open && closeDialog('link')}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{editingItem ? '编辑' : '新增'}关联</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">名称<span className="text-destructive ml-1">*</span></Label>
                <Select
                  value={linkForm.name || ''}
                  onValueChange={(v) => handleFormChange('link', 'name', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="选择名称" />
                  </SelectTrigger>
                  <SelectContent>
                    {infoData.map((item) => (
                      <SelectItem key={item.id} value={item.name}>
                        {item.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="screwSpec">
                  {config.name.includes('皮带') || config.name.includes('模具') || config.name.includes('剪刀') || config.name.includes('上冲') ? '关联线材规格' : '关联螺丝规格'}
                  <span className="text-destructive ml-1">*</span>
                </Label>
                <Select
                  value={linkForm.screwSpec || linkForm.wireSpec || ''}
                  onValueChange={(v) => {
                    const key = config.name.includes('皮带') || config.name.includes('模具') || config.name.includes('剪刀') || config.name.includes('上冲') ? 'wireSpec' : 'screwSpec'
                    handleFormChange('link', key, v)
                  }}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="选择规格" />
                  </SelectTrigger>
                  <SelectContent>
                    {screwSpecs.map((spec) => (
                      <SelectItem key={spec.id} value={spec.name}>
                        {spec.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => closeDialog('link')}>
                取消
              </Button>
              <Button onClick={() => handleSave('link')}>
                保存
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}

      {/* Delete Confirm Dialog */}
      <ConfirmDialog
        open={deleteConfirmOpen}
        onConfirm={handleDelete}
        onCancel={() => {
          setDeleteConfirmOpen(false)
          setDeletingItem(null)
        }}
        title="确认删除"
        description="确定要删除这条记录吗？此操作不可撤销。"
        confirmText="删除"
        cancelText="取消"
      />

      {/* Screw Specs Popup */}
      <Dialog open={screwSpecsPopup.open} onOpenChange={(open) => setScrewSpecsPopup({ ...screwSpecsPopup, open })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>关联螺丝规格</DialogTitle>
          </DialogHeader>
          <div className="py-4">
            {screwSpecsPopup.specs.length > 0 ? (
              <ul className="list-disc list-inside space-y-2">
                {screwSpecsPopup.specs.map((spec, index) => (
                  <li key={index} className="text-sm">
                    {spec}
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-muted-foreground">暂无关联规格</p>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setScrewSpecsPopup({ open: false, specs: [] })}>
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
