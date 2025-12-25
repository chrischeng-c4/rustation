import { useState } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { LayoutDashboard, Container, Settings } from 'lucide-react'
import { DockersPage } from '@/features/dockers/DockersPage'

type Tab = 'workflows' | 'dockers' | 'settings'

function App(): JSX.Element {
  const [activeTab, setActiveTab] = useState<Tab>('dockers')

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <Tabs
        value={activeTab}
        onValueChange={(v) => setActiveTab(v as Tab)}
        orientation="vertical"
        className="flex h-full"
      >
        <TabsList className="flex h-full w-16 flex-col items-center gap-2 rounded-none border-r bg-muted/40 p-2">
          <TabsTrigger
            value="workflows"
            className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
          >
            <LayoutDashboard className="h-5 w-5" />
            <span className="text-[10px]">Flows</span>
          </TabsTrigger>
          <TabsTrigger
            value="dockers"
            className="flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
          >
            <Container className="h-5 w-5" />
            <span className="text-[10px]">Docker</span>
          </TabsTrigger>
          <TabsTrigger
            value="settings"
            className="mt-auto flex h-12 w-12 flex-col items-center justify-center gap-1 rounded-lg"
          >
            <Settings className="h-5 w-5" />
            <span className="text-[10px]">Settings</span>
          </TabsTrigger>
        </TabsList>

        {/* Main Content */}
        <div className="flex-1 overflow-auto p-6">
          <TabsContent value="workflows" className="m-0 h-full">
            <div className="flex h-full items-center justify-center text-muted-foreground">
              Workflows - Coming Soon
            </div>
          </TabsContent>
          <TabsContent value="dockers" className="m-0 h-full">
            <DockersPage />
          </TabsContent>
          <TabsContent value="settings" className="m-0 h-full">
            <div className="flex h-full items-center justify-center text-muted-foreground">
              Settings - Coming Soon
            </div>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}

export default App
