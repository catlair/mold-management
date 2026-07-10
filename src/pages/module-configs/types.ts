import type { ColumnDef } from '@tanstack/react-table'
import type { LucideIcon } from 'lucide-react'

export interface FormField {
  key: string
  label: string
  type: 'text' | 'number' | 'select'
  required?: boolean
  placeholder?: string
  options?: string[]
  defaultValue?: any
}

export interface ModuleConfig {
  name: string
  icon: LucideIcon
  stockType: string
  infoApi: ReturnType<typeof import('@/api').createApi>
  orderApi: ReturnType<typeof import('@/api').createApi>
  useApi: ReturnType<typeof import('@/api').createApi>
  linkApi?: ReturnType<typeof import('@/api').createApi>
  stockApi?: ReturnType<typeof import('@/api').createApi>
  infoColumns: ColumnDef<any, any>[]
  orderColumns: ColumnDef<any, any>[]
  useColumns: ColumnDef<any, any>[]
  linkColumns?: ColumnDef<any, any>[]
  formFields: FormField[]
  orderFormFields?: FormField[]
  useFormFields?: FormField[]
}
