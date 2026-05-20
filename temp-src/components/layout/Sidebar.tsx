import { FlaskConical, LayoutDashboard, FolderOpen, Network, PlayCircle, BookOpen, HelpCircle, Plus } from "lucide-react";

export type ViewType = 'dashboard' | 'projects' | 'explorer' | 'workflows' | 'runs' | 'configure-run';

interface SidebarProps {
  currentView: ViewType;
  setCurrentView: (view: ViewType) => void;
}

export function Sidebar({ currentView, setCurrentView }: SidebarProps) {
  return (
    <nav className="hidden md:flex flex-col w-64 h-screen sticky left-0 top-0 py-4 bg-surface-container-lowest border-r border-outline-variant shrink-0 z-40">
      {/* Header section */}
      <div className="px-4 mb-4">
        <div className="flex items-center gap-2 mb-2">
          <div className="w-8 h-8 rounded bg-primary-container flex items-center justify-center text-on-primary-container">
            <FlaskConical className="w-5 h-5" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-on-surface leading-tight">Project Alpha</h1>
            <div className="font-mono text-xs text-muted-text">v1.2.4-stable</div>
          </div>
        </div>
      </div>

      {/* CTA */}
      <div className="px-4 mb-4">
        <button 
          onClick={() => setCurrentView('configure-run')}
          className="w-full bg-primary hover:bg-primary-fixed-variant text-on-primary text-sm font-semibold rounded-lg py-2 px-4 flex items-center justify-center gap-2 transition-colors active:scale-95 duration-100 shadow-sm"
        >
          <Plus className="w-4 h-4" />
          New Run
        </button>
      </div>

      {/* Main Navigation Tabs */}
      <div className="flex-1 overflow-y-auto px-2 space-y-1">
        <button 
          onClick={() => setCurrentView('dashboard')}
          className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors active:scale-95 duration-100 ${
            currentView === 'dashboard' || currentView === 'projects'
              ? 'bg-secondary-container text-on-secondary-container font-semibold' 
              : 'text-secondary hover:bg-surface-container-high'
          }`}
        >
          <LayoutDashboard className="w-5 h-5" />
          <span>Dashboard</span>
        </button>
        <button 
          onClick={() => setCurrentView('explorer')}
          className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors active:scale-95 duration-100 ${
             currentView === 'explorer'
             ? 'bg-secondary-container text-on-secondary-container font-semibold' 
             : 'text-secondary hover:bg-surface-container-high'
          }`}
        >
          <FolderOpen className="w-5 h-5" />
          <span>Explorer</span>
        </button>
        <button 
          onClick={() => setCurrentView('workflows')}
          className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors active:scale-95 duration-100 ${
             currentView === 'workflows' || currentView === 'configure-run'
             ? 'bg-secondary-container text-on-secondary-container font-semibold' 
             : 'text-secondary hover:bg-surface-container-high'
          }`}
        >
          <Network className="w-5 h-5" />
          <span>Workflows</span>
        </button>
        <button 
          onClick={() => setCurrentView('runs')}
          className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors active:scale-95 duration-100 ${
             currentView === 'runs'
             ? 'bg-secondary-container text-on-secondary-container font-semibold' 
             : 'text-secondary hover:bg-surface-container-high'
          }`}
        >
          <PlayCircle className="w-5 h-5" />
          <span>Runs</span>
        </button>
      </div>

      {/* Footer Navigation Tabs */}
      <div className="px-2 mt-auto pt-4 border-t border-outline-variant/30 space-y-1">
        <div className="px-3 pb-1 text-[11px] font-semibold tracking-wide uppercase text-muted-text">Resources</div>
        <button className="w-full flex items-center gap-2 px-3 py-2 text-secondary hover:bg-surface-container-high transition-colors rounded-lg text-sm text-left">
          <BookOpen className="w-5 h-5" />
          <span>Documentation</span>
        </button>
        <button className="w-full flex items-center gap-2 px-3 py-2 text-secondary hover:bg-surface-container-high transition-colors rounded-lg text-sm text-left">
          <HelpCircle className="w-5 h-5" />
          <span>Support</span>
        </button>
      </div>
    </nav>
  );
}
