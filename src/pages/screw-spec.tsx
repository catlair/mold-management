import { useState, useEffect, useCallback, useMemo } from 'react'
import {
  Plus,
  Maximize2,
  Minimize2,
  Pencil,
  Trash2,
  Eye,
} from 'lucide-react'
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
  type ColumnFiltersState,
  type ColumnResizeMode,
} from '@tanstack/react-table'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { InputNumber } from '@/components/ui/input-number'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { useFullscreen } from '@/hooks/use-fullscreen'
import { cn } from '@/lib/utils'
import { screwSpecApi, punchApi, dieApi, punchLinkApi, dieLinkApi, stockCalcApi } from '@/api'
import { toShortCode } from '@/utils/punch-name'
import { toast } from 'sonner'

// Multi-select checkbox component
function MultiSelectField({
  label,
  value,
  onChange,
  options,
}: {
  label: string
  value: string[]
  onChange: (v: string[]) => void
  options: { label: string; value: string }[]
}) {
  const [open, setOpen] = useState(false)

  const toggleOption = (optValue: string) => {
    if (value.includes(optValue)) {
      onChange(value.filter((v) => v !== optValue))
    } else {
      onChange([...value, optValue])
    }
  }

  const displayText = useMemo(() => {
    if (value.length === 0) return `请选择${label}`
    if (value.length <= 2) return value.join(', ')
    return `${value[0]} 等 ${value.length} 项`
  }, [value, label])

  return (
    <div className="relative">
      <Label className="mb-1 block">{label}</Label>
      <button
        type="button"
        className="flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        onClick={() => setOpen(!open)}
      >
        <span className={value.length === 0 ? 'text-muted-foreground' : ''}>
          {displayText}
        </span>
        <svg
          className={cn('h-4 w-4 opacity-50 transition-transform', open && 'rotate-180')}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {open && (
        <div className="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border bg-popover text-popover-foreground shadow-md">
          {options.map((opt) => (
            <label
              key={opt.value}
              className="flex cursor-pointer items-center px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground"
            >
              <input
                type="checkbox"
                className="mr-2 h-4 w-4"
                checked={value.includes(opt.value)}
                onChange={() => toggleOption(opt.value)}
              />
              {opt.label}
            </label>
          ))}
        </div>
      )}
    </div>
  )
}

// Custom scrollbar component
function CustomScrollbar({
  children,
  className,
  style,
}: {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}) {
  return (
    <div
      className={cn(
        'overflow-auto [&::-webkit-scrollbar]:h-2 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:bg-border [&::-webkit-scrollbar-track]:bg-transparent',
        className
      )}
      style={style}
    >
      {children}
    </div>
  )
}

const statusMap: Record<string, { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  '正常': { label: '正常', variant: 'default' },
  '低库存': { label: '低库存', variant: 'secondary' },
  '缺货': { label: '缺货', variant: 'destructive' },
}

export default function ScrewSpecPage() {
  const { isFullscreen, toggle: toggleFullscreen } = useFullscreen()

  // Data states
  const [screwSpecs, setScrewSpecs] = useState<any[]>([])
  const [punchInfo, setPunchInfo] = useState<any[]>([])
  const [dieInfo, setDieInfo] = useState<any[]>([])
  const [punchLinks, setPunchLinks] = useState<any[]>([])
  const [dieLinks, setDieLinks] = useState<any[]>([])
  const [punchStock, setPunchStock] = useState<any[]>([])
  const [dieStock, setDieStock] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  // Dialog states
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [editingItem, setEditingItem] = useState<any>(null)
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false)
  const [deletingItem, setDeletingItem] = useState<any>(null)

  // Association dialog states
  const [punchDialogOpen, setPunchDialogOpen] = useState(false)
  const [punchDialogSpec, setPunchDialogSpec] = useState<any>(null)
  const [dieDialogOpen, setDieDialogOpen] = useState(false)
  const [dieDialogSpec, setDieDialogSpec] = useState<any>(null)

  // Form state
  const [form, setForm] = useState<Record<string, any>>({})
  const [punchSelected, setPunchSelected] = useState<string[]>([])
  const [dieSelected, setDieSelected] = useState<string[]>([])

  // Sorting and filtering
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])

  // Load all data in parallel
  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const [screws, punches, dies, pLinks, dLinks, pStock, dStock] = await Promise.all([
        screwSpecApi.getAll(),
        punchApi.getAll(),
        dieApi.getAll(),
        punchLinkApi.getAll(),
        dieLinkApi.getAll(),
        stockCalcApi.calculate('punch'),
        stockCalcApi.calculate('die'),
      ])

      const punchInfoArr = Array.isArray(punches) ? punches : []
      const dieInfoArr = Array.isArray(dies) ? dies : []
      const pLinksArr = Array.isArray(pLinks) ? pLinks : []
      const dLinksArr = Array.isArray(dLinks) ? dLinks : []
      const pStockArr = Array.isArray(pStock) ? pStock : []
      const dStockArr = Array.isArray(dStock) ? dStock : []

      setPunchInfo(punchInfoArr)
      setDieInfo(dieInfoArr)
      setPunchLinks(pLinksArr)
      setDieLinks(dLinksArr)
      setPunchStock(pStockArr)
      setDieStock(dStockArr)

      // Resolve link IDs to names and attach to each screw spec
      const screwsArr = Array.isArray(screws) ? screws : []
      const enriched = screwsArr.map((screw: any) => {
        // Find punch links for this screw spec
        const pLinkIds = pLinksArr
          .filter((link: any) => link.screwSpecId === screw.id)
          .map((link: any) => link.punchId)
        const _punchIds = pLinkIds

        // Find die links for this screw spec
        const dLinkIds = dLinksArr
          .filter((link: any) => link.screwSpecId === screw.id)
          .map((link: any) => link.dieId)
        const _dieNames = dLinkIds

        return {
          ...screw,
          _punchIds,
          _dieNames,
        }
      })

      setScrewSpecs(enriched)
    } catch (error) {
      console.error('Failed to load data:', error)
      toast.error('加载数据失败')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadData()
  }, [loadData])

  // Punch options (deduplicated by short code, shows short code + specs)
  const punchOptions = useMemo(() => {
    const seen = new Set<string>()
    return punchInfo
      .map((p: any) => {
        const short = toShortCode(p.name) || p.name
        if (seen.has(short)) return null
        seen.add(short)
        return {
          label: `${short} (${p.spec || ''})`,
          value: p.name,
        }
      })
      .filter(Boolean) as { label: string; value: string }[]
  }, [punchInfo])

  // Die options (deduplicated by name)
  const dieOptions = useMemo(() => {
    const seen = new Set<string>()
    return dieInfo
      .map((d: any) => {
        if (seen.has(d.name)) return null
        seen.add(d.name)
        return {
          label: `${d.name} (${d.machineType || ''} / ${d.wireDiameter || ''})`,
          value: d.name,
        }
      })
      .filter(Boolean) as { label: string; value: string }[]
  }, [dieInfo])

  // Get punch name display with short code
  const getPunchDisplayName = (name: string) => {
    return toShortCode(name) || name
  }

  // Get die name display
  const getDieDisplayName = (name: string) => {
    return name
  }

  // Get unique values for filter columns
  const getUniqueValues = (key: string) => {
    const values = new Set(screwSpecs.map((item) => item[key]).filter(Boolean))
    return Array.from(values)
  }

  // Table columns
  const columns: ColumnDef<any, any>[] = useMemo(
    () => [
      {
        accessorKey: 'name',
        header: '名称',
        size: 160,
        enableSorting: true,
      },
      {
        accessorKey: 'headType',
        header: '头部类型',
        size: 120,
        enableColumnFilter: true,
        filterFn: 'equals',
        meta: {
          filterOptions: getUniqueValues('headType'),
        },
      },
      {
        accessorKey: 'punch',
        header: '冲头',
        size: 120,
        cell: ({ row }) => {
          const name = row.original.punch
          if (!name) return '-'
          const displayName = getPunchDisplayName(name)
          return (
            <button
              className="text-primary hover:underline cursor-pointer text-left font-medium"
              onClick={() => {
                setPunchDialogSpec(row.original)
                setPunchDialogOpen(true)
              }}
            >
              {displayName}
            </button>
          )
        },
      },
      {
        accessorKey: 'threadType',
        header: '牙型',
        size: 120,
        enableColumnFilter: true,
        filterFn: 'equals',
        meta: {
          filterOptions: getUniqueValues('threadType'),
        },
      },
      {
        accessorKey: 'die',
        header: '牙板',
        size: 120,
        cell: ({ row }) => {
          const name = row.original.die
          if (!name) return '-'
          return (
            <button
              className="text-primary hover:underline cursor-pointer text-left font-medium"
              onClick={() => {
                setDieDialogSpec(row.original)
                setDieDialogOpen(true)
              }}
            >
              {getDieDisplayName(name)}
            </button>
          )
        },
      },
      {
        accessorKey: 'headSize',
        header: '头部尺寸',
        size: 140,
      },
      {
        accessorKey: 'headHeight',
        header: '头部高度',
        size: 100,
      },
      {
        accessorKey: 'length',
        header: '长度',
        size: 100,
      },
      {
        accessorKey: 'threadDiameter',
        header: '牙外径',
        size: 100,
      },
      {
        accessorKey: 'shankLength',
        header: '杆长',
        size: 120,
      },
      {
        accessorKey: 'wireMaterial',
        header: '线材材质',
        size: 100,
      },
      {
        accessorKey: 'plating',
        header: '表面处理',
        size: 120,
        enableColumnFilter: true,
        filterFn: 'equals',
        meta: {
          filterOptions: getUniqueValues('plating'),
        },
      },
      {
        accessorKey: 'customer',
        header: '客户',
        size: 120,
      },
      {
        accessorKey: 'externalId',
        header: '外部ID',
        size: 120,
      },
      {
        accessorKey: 'remark',
        header: '备注',
        size: 140,
      },
      {
        id: 'actions',
        header: '操作',
        size: 150,
        enableSorting: false,
        enableColumnFilter: false,
        cell: ({ row }) => (
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => {
                const item = row.original
                setEditingItem(item)
                setForm(item)
                // Set multi-select values from link data
                setPunchSelected(item._punchIds || [])
                setDieSelected(item._dieNames || [])
                setEditDialogOpen(true)
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
    ],
    [screwSpecs, punchInfo, dieInfo]
  )

  const table = useReactTable({
    data: screwSpecs,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    state: {
      sorting,
      columnFilters,
    },
    columnResizeMode: 'fit' as ColumnResizeMode,
  })

  // Handle form field change
  const handleFormChange = (key: string, value: any) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  // Handle save (create/edit)
  const handleSave = async () => {
    // Validate required fields
    if (!form.name) {
      toast.error('请填写名称')
      return
    }

    try {
      const screwData = { ...form }

      if (editingItem) {
        // Update existing screw spec
        await screwSpecApi.update(editingItem.id, screwData)

        // Handle punch links: remove old, add new
        const oldPunchIds = editingItem._punchIds || []
        const newPunchIds = punchSelected

        // Remove old links that are not in new selection
        for (const punchId of oldPunchIds) {
          if (!newPunchIds.includes(punchId)) {
            const link = punchLinks.find(
              (l: any) => l.punchId === punchId && l.screwSpecId === editingItem.id
            )
            if (link) {
              await punchLinkApi.remove(link.id)
            }
          }
        }

        // Add new links that are not in old selection
        for (const punchId of newPunchIds) {
          if (!oldPunchIds.includes(punchId)) {
            await punchLinkApi.add({
              punchId,
              screwSpecId: editingItem.id,
              name: punchInfo.find((p: any) => p.id === punchId)?.name || '',
              screwSpec: form.name,
            })
          }
        }

        // Handle die links: remove old, add new
        const oldDieIds = editingItem._dieNames || []
        const newDieIds = dieSelected

        // Remove old links that are not in new selection
        for (const dieId of oldDieIds) {
          if (!newDieIds.includes(dieId)) {
            const link = dieLinks.find(
              (l: any) => l.dieId === dieId && l.screwSpecId === editingItem.id
            )
            if (link) {
              await dieLinkApi.remove(link.id)
            }
          }
        }

        // Add new links that are not in old selection
        for (const dieId of newDieIds) {
          if (!oldDieIds.includes(dieId)) {
            const dieItem = dieInfo.find((d: any) => d.id === dieId)
            await dieLinkApi.add({
              dieId,
              screwSpecId: editingItem.id,
              name: dieItem?.name || '',
              screwSpec: form.name,
            })
          }
        }

        // Set primary punch/die (first selected or empty)
        screwData.punch = punchSelected.length > 0 ? punchInfo.find((p: any) => p.id === punchSelected[0])?.name || '' : ''
        screwData.die = dieSelected.length > 0 ? dieInfo.find((d: any) => d.id === dieSelected[0])?.name || '' : ''
        await screwSpecApi.update(editingItem.id, { punch: screwData.punch, die: screwData.die })

        toast.success('更新成功')
      } else {
        // Add new screw spec
        const newScrew = await screwSpecApi.add(screwData)
        const newScrewId = newScrew?.id

        if (newScrewId) {
          // Create punch links
          for (const punchId of punchSelected) {
            const punchItem = punchInfo.find((p: any) => p.id === punchId)
            await punchLinkApi.add({
              punchId,
              screwSpecId: newScrewId,
              name: punchItem?.name || '',
              screwSpec: form.name,
            })
          }

          // Create die links
          for (const dieId of dieSelected) {
            const dieItem = dieInfo.find((d: any) => d.id === dieId)
            await dieLinkApi.add({
              dieId,
              screwSpecId: newScrewId,
              name: dieItem?.name || '',
              screwSpec: form.name,
            })
          }

          // Set primary
          const primaryPunch = punchSelected.length > 0 ? punchInfo.find((p: any) => p.id === punchSelected[0])?.name || '' : ''
          const primaryDie = dieSelected.length > 0 ? dieInfo.find((d: any) => d.id === dieSelected[0])?.name || '' : ''
          await screwSpecApi.update(newScrewId, { punch: primaryPunch, die: primaryDie })
        }

        toast.success('新增成功')
      }

      await loadData()
      closeEditDialog()
    } catch (error) {
      console.error('Save failed:', error)
      toast.error('保存失败')
    }
  }

  // Handle delete
  const handleDelete = async () => {
    if (!deletingItem) return

    try {
      // Delete associated links
      const pLinksToDelete = punchLinks.filter((l: any) => l.screwSpecId === deletingItem.id)
      for (const link of pLinksToDelete) {
        await punchLinkApi.remove(link.id)
      }

      const dLinksToDelete = dieLinks.filter((l: any) => l.screwSpecId === deletingItem.id)
      for (const link of dLinksToDelete) {
        await dieLinkApi.remove(link.id)
      }

      await screwSpecApi.remove(deletingItem.id)
      await loadData()
      setDeleteConfirmOpen(false)
      setDeletingItem(null)
      toast.success('删除成功')
    } catch (error) {
      console.error('Delete failed:', error)
      toast.error('删除失败')
    }
  }

  // Close edit dialog
  const closeEditDialog = () => {
    setEditDialogOpen(false)
    setEditingItem(null)
    setForm({})
    setPunchSelected([])
    setDieSelected([])
  }

  // Open add dialog
  const openAddDialog = () => {
    setForm({
      name: '',
      headType: '',
      punch: '',
      threadType: '',
      die: '',
      headSize: '',
      headHeight: '',
      length: '',
      threadDiameter: '',
      shankLength: '',
      wireMaterial: '',
      plating: '',
      customer: '',
      externalId: '',
      remark: '',
    })
    setPunchSelected([])
    setDieSelected([])
    setEditDialogOpen(true)
  }

  // Set primary punch in association dialog
  const handleSetPrimaryPunch = async (screwSpec: any, punchName: string) => {
    try {
      await screwSpecApi.update(screwSpec.id, { punch: punchName })
      await loadData()
      toast.success('已设为主冲头')
    } catch (error) {
      console.error('Failed to set primary punch:', error)
      toast.error('设置失败')
    }
  }

  // Set primary die in association dialog
  const handleSetPrimaryDie = async (screwSpec: any, dieName: string) => {
    try {
      await screwSpecApi.update(screwSpec.id, { die: dieName })
      await loadData()
      toast.success('已设为主牙板')
    } catch (error) {
      console.error('Failed to set primary die:', error)
      toast.error('设置失败')
    }
  }

  // Get linked punches for a screw spec
  const getLinkedPunches = (screwSpec: any) => {
    const linkIds = screwSpec._punchIds || []
    return linkIds
      .map((punchId: string) => {
        const punch = punchInfo.find((p: any) => p.id === punchId)
        if (!punch) return null
        const stock = punchStock.find((s: any) => s.name === punch.name)
        return {
          ...punch,
          currentStock: stock?.currentStock ?? 0,
          status: stock?.status ?? '正常',
        }
      })
      .filter(Boolean)
  }

  // Get linked dies for a screw spec
  const getLinkedDies = (screwSpec: any) => {
    const linkIds = screwSpec._dieNames || []
    return linkIds
      .map((dieId: string) => {
        const die = dieInfo.find((d: any) => d.id === dieId)
        if (!die) return null
        const stock = dieStock.find((s: any) => s.name === die.name)
        return {
          ...die,
          currentStock: stock?.currentStock ?? 0,
          status: stock?.status ?? '正常',
        }
      })
      .filter(Boolean)
  }

  // Form fields for the edit dialog
  const formFields = [
    { key: 'name', label: '名称', type: 'text' as const, required: true },
    { key: 'headType', label: '头部类型', type: 'text' as const },
    { key: 'threadType', label: '牙型', type: 'text' as const },
    { key: 'headSize', label: '头部尺寸', type: 'text' as const },
    { key: 'headHeight', label: '头部高度', type: 'number' as const },
    { key: 'length', label: '长度', type: 'number' as const },
    { key: 'threadDiameter', label: '牙外径', type: 'number' as const },
    { key: 'shankLength', label: '杆长', type: 'number' as const },
    { key: 'wireMaterial', label: '线材材质', type: 'text' as const },
    { key: 'plating', label: '表面处理', type: 'text' as const },
    { key: 'customer', label: '客户', type: 'text' as const },
    { key: 'externalId', label: '外部ID', type: 'text' as const },
    { key: 'remark', label: '备注', type: 'text' as const },
  ]

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
            螺丝规格管理
          </CardTitle>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={openAddDialog}
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
          <CustomScrollbar
            className="rounded-md border"
            style={{
              maxHeight: isFullscreen ? 'calc(100vh - 40px)' : 'calc(100vh - 184px)',
            }}
          >
            <Table>
              <TableHeader>
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow key={headerGroup.id}>
                    {headerGroup.headers.map((header) => (
                      <TableHead key={header.id} colSpan={header.colSpan}>
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                              header.column.columnDef.header,
                              header.getContext()
                            )}
                      </TableHead>
                    ))}
                  </TableRow>
                ))}
              </TableHeader>
              <TableBody>
                {loading ? (
                  Array.from({ length: 5 }).map((_, index) => (
                    <TableRow key={`skeleton-${index}`}>
                      {columns.map((_, colIndex) => (
                        <TableCell key={`skeleton-cell-${colIndex}`}>
                          <div className="h-4 bg-muted animate-pulse rounded" />
                        </TableCell>
                      ))}
                    </TableRow>
                  ))
                ) : table.getRowModel().rows?.length ? (
                  table.getRowModel().rows.map((row) => (
                    <TableRow
                      key={row.id}
                      data-state={row.getIsSelected() && 'selected'}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </TableCell>
                      ))}
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell
                      colSpan={columns.length}
                      className="h-24 text-center"
                    >
                      暂无数据
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </CustomScrollbar>
        </CardContent>
      </Card>

      {/* Edit/Add Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={(open) => !open && closeEditDialog()}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>{editingItem ? '编辑' : '新增'}螺丝规格</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {formFields.map((field) => (
              <div key={field.key} className="grid gap-2">
                <Label htmlFor={field.key}>
                  {field.label}
                  {field.required && <span className="text-destructive ml-1">*</span>}
                </Label>
                {field.type === 'number' ? (
                  <InputNumber
                    value={form[field.key]}
                    onChange={(v) => handleFormChange(field.key, v)}
                    min={0}
                  />
                ) : (
                  <Input
                    value={form[field.key] ?? ''}
                    onChange={(e) => handleFormChange(field.key, e.target.value)}
                    placeholder={`请输入${field.label}`}
                  />
                )}
              </div>
            ))}
            {/* Multi-select punch field */}
            <MultiSelectField
              label="关联冲头"
              value={punchSelected}
              onChange={setPunchSelected}
              options={punchOptions}
            />
            {/* Multi-select die field */}
            <MultiSelectField
              label="关联牙板"
              value={dieSelected}
              onChange={setDieSelected}
              options={dieOptions}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={closeEditDialog}>
              取消
            </Button>
            <Button onClick={handleSave}>
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Punch Association Dialog */}
      <Dialog open={punchDialogOpen} onOpenChange={setPunchDialogOpen}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>
              关联冲头 - {punchDialogSpec ? (toShortCode(punchDialogSpec.name) || punchDialogSpec.name) : ''}
            </DialogTitle>
          </DialogHeader>
          <div className="py-4">
            {punchDialogSpec && (() => {
              const linkedPunches = getLinkedPunches(punchDialogSpec)
              if (linkedPunches.length === 0) {
                return <p className="text-sm text-muted-foreground">暂无关联冲头</p>
              }
              return (
                <CustomScrollbar style={{ maxHeight: '400px' }}>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead style={{ width: 100 }}>名称</TableHead>
                        <TableHead style={{ width: 100 }}>规格</TableHead>
                        <TableHead style={{ width: 80 }}>材质</TableHead>
                        <TableHead style={{ width: 90 }}>当前库存</TableHead>
                        <TableHead style={{ width: 90 }}>安全库存</TableHead>
                        <TableHead style={{ width: 90 }}>状态</TableHead>
                        <TableHead style={{ width: 60 }}></TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {linkedPunches.map((punch: any) => {
                        const isPrimary = punchDialogSpec.punch === punch.name
                        return (
                          <TableRow key={punch.id}>
                            <TableCell>{getPunchDisplayName(punch.name)}</TableCell>
                            <TableCell>{punch.spec}</TableCell>
                            <TableCell>{punch.material}</TableCell>
                            <TableCell>{punch.currentStock}</TableCell>
                            <TableCell>{punch.safetyStock}</TableCell>
                            <TableCell>
                              <Badge variant={statusMap[punch.status]?.variant || 'outline'}>
                                {statusMap[punch.status]?.label || punch.status}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              <button
                                className={cn(
                                  'cursor-pointer p-1 rounded transition-colors',
                                  isPrimary
                                    ? 'text-gray-400'
                                    : 'text-orange-500 hover:text-orange-600'
                                )}
                                onClick={() => handleSetPrimaryPunch(punchDialogSpec, punch.name)}
                                title={isPrimary ? '已是主冲头' : '设为主冲头'}
                              >
                                <Eye className="h-4 w-4" />
                              </button>
                            </TableCell>
                          </TableRow>
                        )
                      })}
                    </TableBody>
                  </Table>
                </CustomScrollbar>
              )
            })()}
          </div>
          <DialogFooter>
            <Button onClick={() => setPunchDialogOpen(false)}>
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Die Association Dialog */}
      <Dialog open={dieDialogOpen} onOpenChange={setDieDialogOpen}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>
              关联牙板 - {dieDialogSpec?.name || ''}
            </DialogTitle>
          </DialogHeader>
          <div className="py-4">
            {dieDialogSpec && (() => {
              const linkedDies = getLinkedDies(dieDialogSpec)
              if (linkedDies.length === 0) {
                return <p className="text-sm text-muted-foreground">暂无关联牙板</p>
              }
              return (
                <CustomScrollbar style={{ maxHeight: '400px' }}>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead style={{ width: 100 }}>名称</TableHead>
                        <TableHead style={{ width: 100 }}>机台类型</TableHead>
                        <TableHead style={{ width: 80 }}>线径</TableHead>
                        <TableHead style={{ width: 90 }}>当前库存</TableHead>
                        <TableHead style={{ width: 90 }}>安全库存</TableHead>
                        <TableHead style={{ width: 90 }}>状态</TableHead>
                        <TableHead style={{ width: 60 }}></TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {linkedDies.map((die: any) => {
                        const isPrimary = dieDialogSpec.die === die.name
                        return (
                          <TableRow key={die.id}>
                            <TableCell>{getDieDisplayName(die.name)}</TableCell>
                            <TableCell>{die.machineType}</TableCell>
                            <TableCell>{die.wireDiameter}</TableCell>
                            <TableCell>{die.currentStock}</TableCell>
                            <TableCell>{die.safetyStock}</TableCell>
                            <TableCell>
                              <Badge variant={statusMap[die.status]?.variant || 'outline'}>
                                {statusMap[die.status]?.label || die.status}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              <button
                                className={cn(
                                  'cursor-pointer p-1 rounded transition-colors',
                                  isPrimary
                                    ? 'text-gray-400'
                                    : 'text-orange-500 hover:text-orange-600'
                                )}
                                onClick={() => handleSetPrimaryDie(dieDialogSpec, die.name)}
                                title={isPrimary ? '已是主牙板' : '设为主牙板'}
                              >
                                <Eye className="h-4 w-4" />
                              </button>
                            </TableCell>
                          </TableRow>
                        )
                      })}
                    </TableBody>
                  </Table>
                </CustomScrollbar>
              )
            })()}
          </div>
          <DialogFooter>
            <Button onClick={() => setDieDialogOpen(false)}>
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirm Dialog */}
      <ConfirmDialog
        open={deleteConfirmOpen}
        onConfirm={handleDelete}
        onCancel={() => {
          setDeleteConfirmOpen(false)
          setDeletingItem(null)
        }}
        title="确认删除"
        description="确定要删除这条螺丝规格吗？关联的冲头和牙板关联也将被删除。此操作不可撤销。"
        confirmText="删除"
        cancelText="取消"
      />
    </div>
  )
}
