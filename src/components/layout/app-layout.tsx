import { Outlet } from 'react-router-dom'
import AppSidebar from './app-sidebar'
import GlobalSearch from '../global-search'
import { useAllowDelete } from '@/hooks/use-allow-delete'


export default function AppLayout() {
  useAllowDelete()

  return (
    <div className="flex h-screen overflow-hidden bg-[hsl(var(--background))]">
      <AppSidebar />
      <main className="flex-1 flex flex-col overflow-hidden">
        <div className="px-6 pt-4 pb-2">
          <GlobalSearch />
        </div>
        <div className="flex-1 overflow-hidden px-6 pb-6">
          <Outlet />
        </div>
      </main>
    </div>
  )
}
