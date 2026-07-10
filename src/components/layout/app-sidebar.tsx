import { NavLink } from 'react-router-dom'
import { useState } from 'react'
import {
  FileText, Wrench, Grid, Link2, Box, Scissors, ArrowUpFromLine,
  Settings, PanelLeftClose, PanelLeftOpen
} from 'lucide-react'
import { cn } from '@/lib/utils'

const navItems = [
  { group: '数据管理', items: [
    { to: '/screw-spec', label: '螺丝规格', icon: FileText },
    { to: '/punch', label: '冲头管理', icon: Wrench },
    { to: '/die', label: '牙板管理', icon: Grid },
    { to: '/belt', label: '皮带管理', icon: Link2 },
    { to: '/main-mold', label: '主模具管理', icon: Box },
    { to: '/scissor', label: '剪刀管理', icon: Scissors },
    { to: '/upper-punch', label: '上冲管理', icon: ArrowUpFromLine },
  ]},
  { group: '系统功能', items: [
    { to: '/settings', label: '配置', icon: Settings },
  ]},
]

export default function AppSidebar() {
  const [collapsed, setCollapsed] = useState(false)

  return (
    <aside className={cn(
      'flex flex-col border-r border-[hsl(var(--border))] bg-[hsl(var(--background))] transition-all duration-300',
      collapsed ? 'w-14' : 'w-44'
    )}>
      {/* Logo */}
      <div className="h-14 flex items-center justify-center border-b border-[hsl(var(--border))] shrink-0">
        {!collapsed && <span className="text-[hsl(var(--primary))] font-bold text-base tracking-wide">模具管理</span>}
      </div>

      {/* Search */}
      {!collapsed && (
        <div className="px-2.5 py-2">
          <div
            className="flex items-center h-8 px-2 rounded-md bg-white/60 border border-[hsl(var(--border))] text-sm text-[hsl(var(--muted-foreground))] cursor-pointer hover:bg-white/80"
            onClick={() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }))}
          >
            <SearchIcon className="w-3.5 h-3.5 mr-1.5 opacity-50" />
            <span>搜索...</span>
          </div>
        </div>
      )}

      {/* Nav */}
      <nav className="flex-1 overflow-y-auto py-2 px-2.5">
        {navItems.map((group) => (
          <div key={group.group} className="mb-3">
            {!collapsed && (
              <div className="px-2.5 py-1 text-[11px] font-semibold text-[hsl(var(--muted-foreground))] uppercase tracking-wider">
                {group.group}
              </div>
            )}
            {group.items.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2.5 h-9 rounded-md text-sm transition-colors mb-0.5',
                    collapsed ? 'justify-center px-0' : 'px-3',
                    isActive
                      ? 'bg-[hsl(var(--primary)/0.1)] text-[hsl(var(--primary))] font-semibold'
                      : 'text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))]'
                  )
                }
              >
                <item.icon className="w-4.5 h-4.5 shrink-0" />
                {!collapsed && <span>{item.label}</span>}
              </NavLink>
            ))}
          </div>
        ))}
      </nav>

      {/* Collapse button */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="h-10 flex items-center justify-center border-t border-[hsl(var(--border))] text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--primary))] transition-colors shrink-0"
      >
        {collapsed ? <PanelLeftOpen className="w-4 h-4" /> : <PanelLeftClose className="w-4 h-4" />}
      </button>
    </aside>
  )
}

function SearchIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
    </svg>
  )
}
