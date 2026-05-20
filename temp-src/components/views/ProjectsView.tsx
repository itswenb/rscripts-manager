import { Filter, ChevronDown, Grid, List as ListIcon, CheckCircle2, RefreshCw, AlertCircle, PauseCircle, Folder, Plus } from "lucide-react";

export function ProjectsView() {
  return (
    <div className="flex-1 overflow-y-auto p-4 md:p-6 bg-background">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <h1 className="text-[30px] font-bold text-on-surface leading-tight tracking-tight">Projects</h1>
          <p className="text-sm text-muted-text mt-1">Manage and monitor all internal research workspaces.</p>
        </div>
        <button className="bg-primary hover:bg-primary-fixed-variant text-on-primary text-sm font-semibold px-4 py-2 rounded-lg transition-colors flex items-center justify-center gap-1.5 shadow-sm whitespace-nowrap active:scale-95 duration-100">
          <Plus className="w-4 h-4" />
          New Project
        </button>
      </div>

      {/* Control Bar */}
      <div className="flex flex-wrap items-center justify-between gap-4 mb-4 bg-surface-container-lowest border border-border rounded-xl p-2 shrink-0">
        <div className="flex items-center gap-2 text-sm">
          <div className="relative flex items-center bg-transparent border-none">
             <Filter className="absolute left-2.5 text-muted-text w-4 h-4" />
             <select className="appearance-none bg-transparent border-none focus:ring-0 text-sm text-on-surface pl-8 pr-8 py-1.5 cursor-pointer outline-none">
               <option>All Statuses</option>
               <option>Running</option>
               <option>Success</option>
               <option>Failed</option>
             </select>
             <ChevronDown className="absolute right-2.5 text-muted-text w-4 h-4 pointer-events-none" />
          </div>
          <div className="w-px h-4 bg-border"></div>
          <div className="relative flex items-center">
             <select className="appearance-none bg-transparent border-none focus:ring-0 text-sm text-on-surface pl-3 pr-8 py-1.5 cursor-pointer outline-none">
               <option>Sort by: Last Modified</option>
               <option>Sort by: Created Date</option>
               <option>Sort by: Name A-Z</option>
             </select>
             <ChevronDown className="absolute right-2.5 text-muted-text w-4 h-4 pointer-events-none" />
          </div>
        </div>
        <div className="flex items-center gap-1 px-2">
           <button className="p-1.5 rounded-md bg-surface-container-high text-on-surface focus:outline-none">
             <Grid className="w-4 h-4" />
           </button>
           <button className="p-1.5 rounded-md text-muted-text hover:bg-surface-container-low transition-colors focus:outline-none">
             <ListIcon className="w-4 h-4" />
           </button>
        </div>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        
        {/* Card 1: Success */}
        <div className="bg-surface-container-lowest border border-border rounded-xl p-4 flex flex-col gap-2 hover:shadow-[0_2px_8px_rgba(0,0,0,0.04)] transition-shadow cursor-pointer group">
          <div className="flex justify-between items-start mb-2">
            <span className="font-mono text-xs text-muted-text bg-surface-container px-1.5 py-0.5 rounded">PRJ-084</span>
            <span className="bg-[#10B9811A] text-success text-[11px] font-bold tracking-wide px-2 py-0.5 rounded-full flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Success
            </span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-on-surface group-hover:text-primary transition-colors leading-snug">Genome Sequencing Alpha</h3>
            <p className="text-sm text-secondary line-clamp-2 mt-1">Primary dataset processing for batch 4. Includes normalizations and variant calling workflows.</p>
          </div>
          <div className="mt-auto pt-4 border-t border-border flex items-center justify-between text-muted-text text-sm">
            <div className="flex items-center gap-1.5">
              <Folder className="w-4 h-4" /> 1,204 files
            </div>
            <span>Updated 2h ago</span>
          </div>
        </div>

        {/* Card 2: Running */}
        <div className="bg-surface-container-lowest border border-primary/30 shadow-[0_0_0_1px_rgba(53,37,205,0.1)] rounded-xl p-4 flex flex-col gap-2 cursor-pointer relative overflow-hidden group">
          <div className="absolute top-0 left-0 h-0.5 bg-primary/20 w-full"><div className="h-full bg-primary w-2/3 animate-pulse"></div></div>
          <div className="flex justify-between items-start mb-2 mt-1">
            <span className="font-mono text-xs text-muted-text bg-surface-container px-1.5 py-0.5 rounded">PRJ-092</span>
            <span className="bg-[#F59E0B1A] text-warning text-[11px] font-bold tracking-wide px-2 py-0.5 rounded-full flex items-center gap-1">
              <RefreshCw className="w-3 h-3 animate-spin" /> Running
            </span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-on-surface group-hover:text-primary transition-colors leading-snug">NLP Sentiment Model v2</h3>
            <p className="text-sm text-secondary line-clamp-2 mt-1">Training job on 500M parameter model across distributed GPU cluster.</p>
          </div>
          <div className="mt-auto pt-4 border-t border-border flex items-center justify-between text-muted-text text-sm">
            <div className="flex items-center gap-1.5">
              <Folder className="w-4 h-4" /> 84 files
            </div>
            <span className="text-warning">Running (45m)</span>
          </div>
        </div>

        {/* Card 3: Error */}
        <div className="bg-surface-container-lowest border border-error-container rounded-xl p-4 flex flex-col gap-2 hover:shadow-[0_2px_8px_rgba(0,0,0,0.04)] transition-shadow cursor-pointer group">
          <div className="flex justify-between items-start mb-2">
            <span className="font-mono text-xs text-muted-text bg-surface-container px-1.5 py-0.5 rounded">PRJ-077</span>
            <span className="bg-error-container text-on-error-container text-[11px] font-bold tracking-wide px-2 py-0.5 rounded-full flex items-center gap-1">
              <AlertCircle className="w-3 h-3" /> Failed
            </span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-on-surface group-hover:text-primary transition-colors leading-snug">Climate Topology Mapping</h3>
            <p className="text-sm text-secondary line-clamp-2 mt-1">Data ingestion pipeline failed at step 4 (Out of Memory). Requires immediate intervention.</p>
          </div>
          <div className="mt-auto pt-4 border-t border-border flex items-center justify-between text-muted-text text-sm">
            <div className="flex items-center gap-1.5">
              <Folder className="w-4 h-4" /> 3,492 files
            </div>
            <span className="text-error">Failed Yesterday</span>
          </div>
        </div>

        {/* Card 4: Idle */}
        <div className="bg-surface-container-lowest border border-border rounded-xl p-4 flex flex-col gap-2 hover:shadow-[0_2px_8px_rgba(0,0,0,0.04)] transition-shadow cursor-pointer group">
          <div className="flex justify-between items-start mb-2">
            <span className="font-mono text-xs text-muted-text bg-surface-container px-1.5 py-0.5 rounded">PRJ-095</span>
            <span className="bg-surface-container-high text-on-surface-variant text-[11px] font-bold tracking-wide px-2 py-0.5 rounded-full flex items-center gap-1">
              <PauseCircle className="w-3 h-3" /> Idle
            </span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-on-surface group-hover:text-primary transition-colors leading-snug">Financial Risk Simulator</h3>
            <p className="text-sm text-secondary line-clamp-2 mt-1">Workspace initialized. Awaiting raw data upload from core systems before executing scripts.</p>
          </div>
          <div className="mt-auto pt-4 border-t border-border flex items-center justify-between text-muted-text text-sm">
            <div className="flex items-center gap-1.5">
              <Folder className="w-4 h-4" /> 2 files
            </div>
            <span>Created 3d ago</span>
          </div>
        </div>

        {/* Card 5: Success */}
        <div className="bg-surface-container-lowest border border-border rounded-xl p-4 flex flex-col gap-2 hover:shadow-[0_2px_8px_rgba(0,0,0,0.04)] transition-shadow cursor-pointer group">
          <div className="flex justify-between items-start mb-2">
            <span className="font-mono text-xs text-muted-text bg-surface-container px-1.5 py-0.5 rounded">PRJ-062</span>
            <span className="bg-[#10B9811A] text-success text-[11px] font-bold tracking-wide px-2 py-0.5 rounded-full flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Success
            </span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-on-surface group-hover:text-primary transition-colors leading-snug">Customer Churn Analytics</h3>
            <p className="text-sm text-secondary line-clamp-2 mt-1">Completed historical data analysis spanning 2020-2023. Reports generated in output dir.</p>
          </div>
          <div className="mt-auto pt-4 border-t border-border flex items-center justify-between text-muted-text text-sm">
            <div className="flex items-center gap-1.5">
              <Folder className="w-4 h-4" /> 156 files
            </div>
            <span>Updated 1w ago</span>
          </div>
        </div>

      </div>
    </div>
  );
}
