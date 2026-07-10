import { Routes, Route, Navigate } from 'react-router-dom'
import { Toaster } from 'sonner'
import AppLayout from './components/layout/app-layout'
import ScrewSpecPage from './pages/screw-spec'
import ModulePage from './pages/module-page'
import SettingsPage from './pages/settings'
import NotFound from './pages/not-found'
import { punchConfig, dieConfig, beltConfig, mainMoldConfig, scissorConfig, upperPunchConfig } from './pages/module-configs'

export default function App() {
  return (
    <>
      <Routes>
        <Route path="/" element={<AppLayout />}>
          <Route index element={<Navigate to="/screw-spec" replace />} />
          <Route path="screw-spec" element={<ScrewSpecPage />} />
          <Route path="punch" element={<ModulePage config={punchConfig} />} />
          <Route path="die" element={<ModulePage config={dieConfig} />} />
          <Route path="belt" element={<ModulePage config={beltConfig} />} />
          <Route path="main-mold" element={<ModulePage config={mainMoldConfig} />} />
          <Route path="scissor" element={<ModulePage config={scissorConfig} />} />
          <Route path="upper-punch" element={<ModulePage config={upperPunchConfig} />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="*" element={<NotFound />} />
        </Route>
      </Routes>
      <Toaster richColors position="top-right" />
    </>
  )
}
