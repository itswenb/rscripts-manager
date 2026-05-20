import { useState } from 'react';
import { Sidebar, ViewType } from './components/layout/Sidebar';
import { TopBar } from './components/layout/TopBar';
import { DashboardView } from './components/views/DashboardView';
import { ProjectsView } from './components/views/ProjectsView';
import { FileExplorerView } from './components/views/FileExplorerView';
import { ExecutionDetailView } from './components/views/ExecutionDetailView';
import { ConfigureRunView } from './components/views/ConfigureRunView';

export default function App() {
  const [currentView, setCurrentView] = useState<ViewType>('configure-run');

  const renderView = () => {
    switch (currentView) {
      case 'dashboard':
        return <DashboardView />;
      case 'projects': // using projects as alias for now, or could show dashboard
        return <ProjectsView />;
      case 'explorer':
        return <FileExplorerView />;
      case 'workflows':
        return <ProjectsView />; // For demo, map workflows to projects view or another overview
      case 'runs':
        return <ExecutionDetailView />;
      case 'configure-run':
        return <ConfigureRunView />;
      default:
        return <DashboardView />;
    }
  };

  return (
    <div className="flex h-screen w-full bg-background text-on-background overflow-hidden selection:bg-primary-container selection:text-on-primary-container">
      <Sidebar currentView={currentView} setCurrentView={setCurrentView} />
      <div className="flex-1 flex flex-col min-w-0 bg-surface">
        <TopBar currentView={currentView} />
        {renderView()}
      </div>
    </div>
  );
}

