import { useState, useEffect, useCallback } from 'react'
import {
  Download,
  Upload,
  FolderOpen,
  RotateCcw,
  Shield,
  Save,
  Undo2,
  Lock,
  Unlock,
  RefreshCw,
  FileDown,
  Folder,
} from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { InputNumber } from '@/components/ui/input-number'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { useAllowDelete } from '@/hooks/use-allow-delete'
import { dataApi, settingsApi, backupApi } from '@/api'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { toast } from 'sonner'

function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

export default function SettingsPage() {
  const { allowDelete, set: setAllowDelete } = useAllowDelete()

  // Data path
  const [dataPath, setDataPath] = useState('')
  const [dataPathLoading, setDataPathLoading] = useState(false)

  // Backup config
  const [backupCount, setBackupCount] = useState<number | undefined>(3)
  const [backupDir, setBackupDir] = useState('')
  const [backupConfigLoading, setBackupConfigLoading] = useState(false)

  // Backup list
  const [backupList, setBackupList] = useState<any[]>([])
  const [backupListLoading, setBackupListLoading] = useState(false)

  // Confirm dialogs
  const [importConfirmOpen, setImportConfirmOpen] = useState(false)
  const [pendingImportData, setPendingImportData] = useState<string>('')
  const [pendingImportFilename, setPendingImportFilename] = useState('')
  const [restoreConfirmOpen, setRestoreConfirmOpen] = useState(false)
  const [pendingRestorePath, setPendingRestorePath] = useState('')

  // Loading states
  const [exporting, setExporting] = useState(false)
  const [importing, setImporting] = useState(false)
  const [backingUp, setBackingUp] = useState(false)

  // Load data path
  const loadDataPath = useCallback(async () => {
    setDataPathLoading(true)
    try {
      const path = await settingsApi.getDataPath()
      setDataPath(path || '')
    } catch (error) {
      console.error('Failed to load data path:', error)
    } finally {
      setDataPathLoading(false)
    }
  }, [])

  // Load backup config
  const loadBackupConfig = useCallback(async () => {
    setBackupConfigLoading(true)
    try {
      const config = await backupApi.getConfig()
      if (config) {
        setBackupCount(config.backupCount ?? 3)
        setBackupDir(config.backupPath || '')
      }
    } catch (error) {
      console.error('Failed to load backup config:', error)
    } finally {
      setBackupConfigLoading(false)
    }
  }, [])

  // Load backup list
  const loadBackupList = useCallback(async () => {
    setBackupListLoading(true)
    try {
      const list = await backupApi.list()
      setBackupList(Array.isArray(list) ? list : [])
    } catch (error) {
      console.error('Failed to load backup list:', error)
    } finally {
      setBackupListLoading(false)
    }
  }, [])

  useEffect(() => {
    loadDataPath()
    loadBackupConfig()
    loadBackupList()
  }, [loadDataPath, loadBackupConfig, loadBackupList])

  // Handle export
  const handleExport = async () => {
    setExporting(true)
    try {
      const result = await dataApi.exportData()
      if (result && result.filename && result.data) {
        const filePath = await save({
          defaultPath: result.filename,
          filters: [{ name: 'Excel', extensions: ['xlsx'] }],
        })
        if (filePath) {
          const bytes = base64ToUint8Array(result.data)
          await writeFile(filePath, bytes)
          toast.success('导出成功')
        }
      }
    } catch (error) {
      console.error('Export failed:', error)
      toast.error('导出失败')
    } finally {
      setExporting(false)
    }
  }

  // Handle import file selection
  const handleImport = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Excel', extensions: ['xlsx'] }],
      })
      if (!selected) return

      const filePath = typeof selected === 'string' ? selected : selected
      const content = await readFile(filePath)
      const base64 = btoa(String.fromCharCode(...content))
      const filename = typeof filePath === 'string' ? filePath.split(/[/\\]/).pop() || 'data.xlsx' : 'data.xlsx'

      setPendingImportData(base64)
      setPendingImportFilename(filename)
      setImportConfirmOpen(true)
    } catch (error) {
      console.error('Import failed:', error)
      toast.error('导入失败')
    }
  }

  // Confirm import
  const confirmImport = async () => {
    setImporting(true)
    try {
      await dataApi.importData(pendingImportData)
      toast.success('导入成功，页面即将刷新')
      setImportConfirmOpen(false)
      setPendingImportData('')
      setPendingImportFilename('')
      setTimeout(() => window.location.reload(), 1000)
    } catch (error) {
      console.error('Import failed:', error)
      toast.error('导入失败')
    } finally {
      setImporting(false)
    }
  }

  // Handle select new data path
  const handleSelectDataPath = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Excel', extensions: ['xlsx'] }],
      })
      if (!selected) return

      const filePath = typeof selected === 'string' ? selected : selected
      await settingsApi.setDataPath(filePath)
      setDataPath(filePath)
      toast.success('数据文件路径已更新')
    } catch (error) {
      console.error('Failed to set data path:', error)
      toast.error('设置数据文件路径失败')
    }
  }

  // Handle toggle allow delete
  const handleToggleDelete = async (checked: boolean) => {
    try {
      await setAllowDelete(checked)
      toast.success(checked ? '已开启删除权限' : '已关闭删除权限')
    } catch (error) {
      console.error('Failed to toggle delete:', error)
      toast.error('切换删除权限失败')
    }
  }

  // Handle save backup config
  const handleSaveBackupConfig = async () => {
    if (backupCount === undefined || backupCount < 1) {
      toast.error('请输入有效的备份数量')
      return
    }
    try {
      await backupApi.setConfig(backupCount, backupDir || null)
      toast.success('备份配置已保存')
    } catch (error) {
      console.error('Failed to save backup config:', error)
      toast.error('保存备份配置失败')
    }
  }

  // Handle select backup directory
  const handleSelectBackupDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })
      if (!selected) return

      const dirPath = typeof selected === 'string' ? selected : selected
      setBackupDir(dirPath)
    } catch (error) {
      console.error('Failed to select backup dir:', error)
      toast.error('选择备份目录失败')
    }
  }

  // Handle reset backup directory
  const handleResetBackupDir = () => {
    setBackupDir('')
  }

  // Handle manual backup
  const handleBackup = async () => {
    setBackingUp(true)
    try {
      await backupApi.backup()
      toast.success('手动备份成功')
      await loadBackupList()
    } catch (error) {
      console.error('Backup failed:', error)
      toast.error('手动备份失败')
    } finally {
      setBackingUp(false)
    }
  }

  // Handle toggle lock
  const handleToggleLock = async (index: number) => {
    try {
      await backupApi.toggleLock(index)
      toast.success('锁定状态已切换')
      await loadBackupList()
    } catch (error) {
      console.error('Failed to toggle lock:', error)
      toast.error('切换锁定状态失败')
    }
  }

  // Handle restore
  const handleRestore = async (backupPath: string) => {
    setPendingRestorePath(backupPath)
    setRestoreConfirmOpen(true)
  }

  // Confirm restore
  const confirmRestore = async () => {
    try {
      await backupApi.restore(pendingRestorePath)
      toast.success('备份恢复成功，页面即将刷新')
      setRestoreConfirmOpen(false)
      setPendingRestorePath('')
      setTimeout(() => window.location.reload(), 1000)
    } catch (error) {
      console.error('Restore failed:', error)
      toast.error('备份恢复失败')
    }
  }

  return (
    <div className="flex flex-col gap-6 p-6 max-w-4xl mx-auto">
      {/* Section 1: Data Import/Export */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileDown className="h-5 w-5" />
            数据导入导出
          </CardTitle>
          <CardDescription>
            导出当前数据为 Excel 文件，或从 Excel 文件导入数据
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <Button
              variant="outline"
              onClick={handleExport}
              disabled={exporting}
            >
              <Download className="h-4 w-4 mr-2" />
              {exporting ? '导出中...' : '导出数据'}
            </Button>
            <Button
              variant="outline"
              onClick={handleImport}
              disabled={importing}
            >
              <Upload className="h-4 w-4 mr-2" />
              {importing ? '导入中...' : '导入数据'}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Section 2: Data File Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Folder className="h-5 w-5" />
            数据文件配置
          </CardTitle>
          <CardDescription>
            当前数据文件的存储路径
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <Label>当前路径</Label>
              <div className="mt-1 p-2 rounded-md bg-muted text-sm font-mono break-all">
                {dataPathLoading ? '加载中...' : (dataPath || '未配置')}
              </div>
            </div>
            <Button
              variant="outline"
              onClick={handleSelectDataPath}
              disabled={dataPathLoading}
            >
              <FolderOpen className="h-4 w-4 mr-2" />
              选择文件
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Section 3: Security Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            安全设置
          </CardTitle>
          <CardDescription>
            控制是否允许删除操作
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <Label>允许删除</Label>
              <p className="text-sm text-muted-foreground">
                开启后允许在各模块中删除数据记录
              </p>
            </div>
            <Switch
              checked={allowDelete}
              onCheckedChange={handleToggleDelete}
            />
          </div>
        </CardContent>
      </Card>

      {/* Section 4: Auto Backup Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Save className="h-5 w-5" />
            自动备份配置
          </CardTitle>
          <CardDescription>
            配置自动备份的数量和存储目录
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            <div className="w-48">
              <Label>备份数量</Label>
              <InputNumber
                value={backupCount}
                onChange={setBackupCount}
                min={1}
                max={99}
                className="mt-1"
                disabled={backupConfigLoading}
              />
            </div>
            <div className="flex-1">
              <Label>备份目录</Label>
              <div className="mt-1 flex items-center gap-2">
                <div className="flex-1 p-2 rounded-md bg-muted text-sm font-mono break-all">
                  {backupDir || '使用默认目录'}
                </div>
                <Button variant="outline" size="sm" onClick={handleSelectBackupDir}>
                  <FolderOpen className="h-4 w-4 mr-1" />
                  更改
                </Button>
                <Button variant="outline" size="sm" onClick={handleResetBackupDir}>
                  <RotateCcw className="h-4 w-4 mr-1" />
                  重置
                </Button>
              </div>
            </div>
          </div>
          <div className="flex gap-2">
            <Button onClick={handleSaveBackupConfig} disabled={backupConfigLoading}>
              保存配置
            </Button>
            <Button
              variant="outline"
              onClick={handleBackup}
              disabled={backingUp}
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              {backingUp ? '备份中...' : '手动备份'}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Section 5: Backup Records */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Undo2 className="h-5 w-5" />
            备份记录
          </CardTitle>
          <CardDescription>
            查看所有备份记录，可锁定或恢复指定备份
          </CardDescription>
        </CardHeader>
        <CardContent>
          {backupListLoading ? (
            <div className="py-8 text-center text-muted-foreground">加载中...</div>
          ) : backupList.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">暂无备份记录</div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>备份时间</TableHead>
                  <TableHead>备份原因</TableHead>
                  <TableHead>MD5</TableHead>
                  <TableHead className="w-20">锁定</TableHead>
                  <TableHead className="w-20">恢复</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {backupList.map((backup, index) => (
                  <TableRow key={index}>
                    <TableCell className="font-mono text-sm">
                      {backup.backup_time || '-'}
                    </TableCell>
                    <TableCell>{backup.backup_reason || '-'}</TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {backup.backup_md5 ? backup.backup_md5.substring(0, 16) + '...' : '-'}
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleToggleLock(index)}
                        title={backup.locked ? '解锁' : '锁定'}
                      >
                        {backup.locked ? (
                          <Lock className="h-4 w-4 text-primary" />
                        ) : (
                          <Unlock className="h-4 w-4 text-muted-foreground" />
                        )}
                      </Button>
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRestore(backup.backup_path || '')}
                        title="恢复此备份"
                      >
                        <RotateCcw className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Import Confirm Dialog */}
      <AlertDialog open={importConfirmOpen} onOpenChange={setImportConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认导入</AlertDialogTitle>
            <AlertDialogDescription>
              确定要导入文件 "{pendingImportFilename}" 吗？导入操作将替换当前所有数据，此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={importing}>取消</AlertDialogCancel>
            <AlertDialogAction onClick={confirmImport} disabled={importing}>
              {importing ? '导入中...' : '确认导入'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Restore Confirm Dialog */}
      <AlertDialog open={restoreConfirmOpen} onOpenChange={setRestoreConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认恢复</AlertDialogTitle>
            <AlertDialogDescription>
              确定要恢复此备份吗？恢复操作将替换当前所有数据，此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={confirmRestore}>
              确认恢复
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
