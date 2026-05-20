import { Search, Bell, HelpCircle, Settings, Menu } from "lucide-react";
import { ViewType } from "./Sidebar";

interface TopBarProps {
  currentView: ViewType;
}

export function TopBar({ currentView }: TopBarProps) {
  return (
    <header className="flex items-center justify-between px-6 h-16 w-full sticky top-0 z-30 bg-surface border-b border-outline-variant shrink-0">
      <div className="flex items-center gap-4 w-full max-w-2xl">
        <button className="md:hidden text-on-surface-variant hover:bg-surface-container-low transition-colors active:opacity-80 p-2 rounded">
          <Menu className="w-5 h-5" />
        </button>
        <div className="text-xl font-bold text-primary flex-shrink-0">
          Rflow
        </div>

        {/* Search Bar */}
        <div className="flex-1 relative max-w-md hidden sm:block ml-4">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-text w-4 h-4" />
          <input 
            type="text" 
            placeholder={currentView === 'explorer' ? "Search workspace..." : "Search files, runs, workflows..."}
            className="w-full bg-surface-container-low border border-outline-variant rounded-lg px-9 py-1.5 text-sm text-on-surface border focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/20 transition-all placeholder:text-muted-text h-8 sm:h-auto"
          />
          <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
            <kbd className="font-mono text-[10px] bg-surface border border-outline-variant rounded px-1.5 py-0.5 text-muted-text">⌘</kbd>
            <kbd className="font-mono text-[10px] bg-surface border border-outline-variant rounded px-1.5 py-0.5 text-muted-text">K</kbd>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button className="w-8 h-8 flex items-center justify-center rounded-full text-on-surface-variant hover:bg-surface-container-low active:opacity-80 transition-colors">
          <Bell className="w-5 h-5" />
        </button>
        <button className="w-8 h-8 flex items-center justify-center rounded-full text-on-surface-variant hover:bg-surface-container-low active:opacity-80 transition-colors">
          <HelpCircle className="w-5 h-5" />
        </button>
        <button className="w-8 h-8 flex items-center justify-center rounded-full text-on-surface-variant hover:bg-surface-container-low active:opacity-80 transition-colors">
          <Settings className="w-5 h-5" />
        </button>
        <div className="w-px h-6 bg-outline-variant mx-1 hidden sm:block"></div>
        <button className="w-8 h-8 rounded-full overflow-hidden border border-outline-variant hover:ring-2 hover:ring-primary/20 transition-all ml-1 bg-surface-container-highest">
          <img 
            src="https://api.dicebear.com/7.x/avataaars/svg?seed=Felix&backgroundColor=e2e8f0" 
            alt="User profile" 
            className="w-full h-full object-cover"
          />
        </button>
      </div>
    </header>
  );
}
