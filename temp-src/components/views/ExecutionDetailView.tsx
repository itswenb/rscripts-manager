import { Network, StopCircle, MoreHorizontal, Download, Maximize, Timer, Server } from "lucide-react";

export function ExecutionDetailView() {
  return (
    <div className="flex-1 flex flex-col p-4 md:p-6 gap-6 overflow-hidden bg-surface-bright min-h-0">
      
      {/* Breadcrumb & Header */}
      <div className="flex flex-col gap-1 shrink-0">
        <div className="text-[13px] text-on-surface-variant flex items-center gap-1.5">
          <Network className="w-4 h-4" />
          <span>DataPipeline_v2</span>
          <span className="text-outline">/</span>
          <span className="font-mono text-on-surface tracking-tight">Run-9a2f-48bc</span>
        </div>
        
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mt-2">
          <h2 className="text-2xl lg:text-3xl font-bold text-on-surface flex flex-wrap items-center gap-3">
            Execution: DataPipeline_v2
            <span className="px-2 py-1 rounded bg-surface-container-low border border-outline-variant font-mono text-sm text-on-surface-variant font-medium mt-1 sm:mt-0">#9a2f-48bc</span>
          </h2>
          <div className="flex items-center gap-2">
            <button className="px-4 py-2 bg-surface text-on-surface border border-outline-variant rounded hover:bg-surface-container-low transition-colors text-sm font-semibold flex items-center gap-1.5 shadow-sm active:scale-95 duration-100">
               <StopCircle className="w-4 h-4" /> Abort
            </button>
            <button className="px-3 py-2 bg-surface text-on-surface border border-outline-variant rounded hover:bg-surface-container-low transition-colors text-sm font-semibold flex items-center gap-1.5 shadow-sm active:scale-95 duration-100">
               <MoreHorizontal className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      {/* Metrics Bento Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 shrink-0">
        <div className="bg-surface-container-lowest border border-border rounded-lg p-4 flex flex-col justify-between shadow-sm">
           <span className="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant mb-2">Status</span>
           <div className="flex items-center gap-2">
             <span className="relative flex h-2.5 w-2.5">
               <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
               <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-primary"></span>
             </span>
             <span className="text-lg font-bold text-primary">Running</span>
           </div>
        </div>
        <div className="bg-surface-container-lowest border border-border rounded-lg p-4 flex flex-col justify-between shadow-sm">
           <span className="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant mb-2">Start Time</span>
           <div className="font-mono text-[13px] text-on-surface">2023-10-27 14:30:05 UTC</div>
        </div>
        <div className="bg-surface-container-lowest border border-border rounded-lg p-4 flex flex-col justify-between shadow-sm">
           <span className="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant mb-2">Duration</span>
           <div className="font-mono text-[13px] text-on-surface flex items-center gap-1.5">
             <Timer className="w-4 h-4 text-on-surface-variant" /> 00:04:12
           </div>
        </div>
        <div className="bg-surface-container-lowest border border-border rounded-lg p-4 flex flex-col justify-between shadow-sm">
           <span className="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant mb-2">Compute Node</span>
           <div className="font-mono text-[13px] text-on-surface flex items-center gap-1.5">
             <Server className="w-4 h-4 text-on-surface-variant" /> worker-gpu-04
           </div>
        </div>
      </div>

      {/* Progress Section */}
      <div className="bg-surface-container-lowest border border-border rounded-lg p-4 shrink-0 shadow-sm">
        <div className="flex justify-between items-end mb-3">
          <div>
            <div className="text-[11px] font-bold uppercase tracking-wider text-primary mb-1">Step 3 of 5</div>
            <div className="text-lg font-bold text-on-surface">Data Transformation (MapReduce)</div>
          </div>
          <div className="font-mono text-sm text-secondary">60%</div>
        </div>
        <div className="w-full bg-surface-container-high rounded-full h-2 overflow-hidden">
          <div className="bg-primary h-2 rounded-full transition-all duration-500 ease-in-out w-[60%]"></div>
        </div>
      </div>

      {/* Main Content Area (Tabs + Logs) */}
      <div className="flex-1 flex flex-col min-h-0 bg-surface-container-lowest border border-border rounded-lg overflow-hidden shadow-sm">
        {/* Tabs Header */}
        <div className="flex items-center px-2 border-b border-border bg-surface shrink-0 overflow-x-auto">
          <button className="px-4 py-3 text-sm font-bold text-primary border-b-2 border-primary whitespace-nowrap">
            Live Logs
          </button>
          <button className="px-4 py-3 text-sm font-medium text-on-surface-variant hover:text-on-surface transition-colors whitespace-nowrap">
            Results
          </button>
          <button className="px-4 py-3 text-sm font-medium text-on-surface-variant hover:text-on-surface transition-colors whitespace-nowrap">
            Configuration
          </button>
          
          <div className="ml-auto flex items-center gap-2 px-2 shrink-0">
             <label className="flex items-center gap-1.5 text-sm text-on-surface-variant cursor-pointer">
               <input type="checkbox" defaultChecked className="rounded-[2px] border-outline-variant text-primary focus:ring-primary w-3.5 h-3.5 bg-transparent" />
               Auto-scroll
             </label>
             <div className="w-px h-4 bg-border mx-1"></div>
             <button className="text-on-surface-variant hover:text-on-surface transition-colors p-1.5 rounded hover:bg-surface-container-low" title="Download Logs">
               <Download className="w-4 h-4" />
             </button>
             <button className="text-on-surface-variant hover:text-on-surface transition-colors p-1.5 rounded hover:bg-surface-container-low" title="Fullscreen">
               <Maximize className="w-4 h-4" />
             </button>
          </div>
        </div>

        {/* Log Viewer */}
        <div className="flex-1 bg-[#1e2330] text-[#e2e8f0] overflow-auto p-4 font-mono text-[12px] leading-relaxed">
           <div className="flex flex-col gap-1">
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:05.102</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Initializing environment variables...</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:05.441</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Connecting to cluster worker-gpu-04...</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:06.012</span>
                <span className="text-[#10b981] shrink-0 w-12">[OK]</span>
                <span className="text-[#f8fafc]">Connection established. Allocation: 4 GPUs, 64GB RAM.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:07.120</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Step 1: Downloading dataset 'gs://bucket-alpha/raw_data.csv' (1.2GB)</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:15.890</span>
                <span className="text-[#10b981] shrink-0 w-12">[OK]</span>
                <span className="text-[#f8fafc]">Download complete. Checksum verified.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:16.002</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Step 2: Preprocessing pipeline started. Modules: [clean_nulls, normalize_features]</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:22.314</span>
                <span className="text-[#f59e0b] shrink-0 w-12">[WARN]</span>
                <span className="text-[#f59e0b]">clean_nulls module detected &gt;5% missing values in column 'sensor_3'. Proceeding with mean imputation.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:45.991</span>
                <span className="text-[#10b981] shrink-0 w-12">[OK]</span>
                <span className="text-[#f8fafc]">Preprocessing complete. Shape: (1048576, 24)</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:30:46.005</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Step 3: Data Transformation (MapReduce) initialized. Batch size: 1024.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:31:10.550</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Batch 100/1024 processed. Rate: 8.4 batches/sec.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:32:00.120</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Batch 500/1024 processed. Rate: 8.2 batches/sec.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded hover:bg-white/5">
                <span className="text-[#94a3b8] shrink-0">14:33:05.882</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Batch 800/1024 processed. Rate: 8.3 batches/sec.</span>
              </div>
              <div className="flex gap-3 px-2 py-0.5 rounded bg-primary/20 animate-pulse">
                <span className="text-[#94a3b8] shrink-0">14:34:12.001</span>
                <span className="text-[#818cf8] shrink-0 w-12">[INFO]</span>
                <span className="text-[#f8fafc]">Processing remaining batches...</span>
              </div>
           </div>
        </div>
      </div>

    </div>
  );
}
