import { Play, UploadCloud, MoreHorizontal, FileText, CheckCircle2, RefreshCw, AlertCircle } from "lucide-react";

export function DashboardView() {
  return (
    <div className="flex-1 overflow-y-auto p-4 md:p-6 bg-background">
      <div className="max-w-7xl mx-auto space-y-6">
        
        {/* ROW 1 */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Main Project Status Card */}
          <div className="lg:col-span-8 bg-surface border border-border rounded-xl p-4 md:p-6 flex flex-col justify-between shadow-[0_1px_2px_rgba(0,0,0,0.05)] relative overflow-hidden group">
            <div className="absolute right-0 top-0 w-64 h-64 bg-primary/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/4 pointer-events-none"></div>
            
            <div className="relative z-10 flex items-start justify-between mb-8">
              <div>
                <h2 className="text-2xl font-bold text-on-surface mb-1">Project Alpha Overview</h2>
                <p className="text-sm text-secondary">Genomics sequencing data pipeline. Currently processing batch 42.</p>
              </div>
              <div className="px-2 py-1 rounded bg-success/10 text-success text-[11px] font-semibold tracking-wide uppercase flex items-center gap-1.5 border border-success/20">
                <span className="w-1.5 h-1.5 rounded-full bg-success"></span>
                System Healthy
              </div>
            </div>
            
            <div className="relative z-10 grid grid-cols-3 gap-4 border-t border-border pt-4">
              <div>
                <div className="text-[11px] font-bold tracking-wide uppercase text-muted-text mb-1">Active Workflows</div>
                <div className="text-xl md:text-2xl font-bold text-on-surface">12</div>
              </div>
              <div className="border-l border-border pl-4">
                <div className="text-[11px] font-bold tracking-wide uppercase text-muted-text mb-1">Failed Runs (24h)</div>
                <div className="text-xl md:text-2xl font-bold text-on-surface">0</div>
              </div>
              <div className="border-l border-border pl-4">
                <div className="text-[11px] font-bold tracking-wide uppercase text-muted-text mb-1">Compute Hours</div>
                <div className="text-xl md:text-2xl font-bold text-on-surface">1,240 <span className="text-sm text-muted-text font-normal">hrs</span></div>
              </div>
            </div>
          </div>

          {/* Quick Actions Card */}
          <div className="lg:col-span-4 bg-surface border border-border rounded-xl p-4 md:p-6 shadow-[0_1px_2px_rgba(0,0,0,0.05)] flex flex-col">
            <h3 className="text-[11px] font-bold tracking-wide uppercase text-muted-text mb-4">Quick Actions</h3>
            <div className="space-y-3 flex-1 flex flex-col justify-center">
              <button className="w-full group relative flex items-center p-3 rounded-lg border border-primary/20 bg-primary/5 hover:bg-primary/10 transition-colors text-left">
                <div className="w-10 h-10 rounded bg-surface border border-border flex items-center justify-center text-primary mr-3 shadow-sm">
                  <Play className="w-5 h-5 fill-current" />
                </div>
                <div className="flex-1">
                  <div className="text-sm font-semibold text-on-surface group-hover:text-primary transition-colors">Start New Run</div>
                  <div className="text-xs text-secondary">Execute primary pipeline script</div>
                </div>
              </button>
              
              <button className="w-full group relative flex items-center p-3 rounded-lg border border-border bg-surface hover:bg-surface-container-low transition-colors text-left">
                <div className="w-10 h-10 rounded bg-surface border border-border flex items-center justify-center text-secondary mr-3 shadow-sm">
                  <UploadCloud className="w-5 h-5" />
                </div>
                <div className="flex-1">
                  <div className="text-sm font-semibold text-on-surface">Upload Datasets</div>
                  <div className="text-xs text-secondary">Add raw files to storage</div>
                </div>
              </button>
            </div>
          </div>
        </div>

        {/* ROW 2 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* ScriptRuns Chart */}
          <div className="bg-surface border border-border rounded-xl p-4 md:p-6 shadow-[0_1px_2px_rgba(0,0,0,0.05)] flex flex-col h-64">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-[11px] font-bold tracking-wide uppercase text-muted-text">ScriptRuns (Last 7 Days)</h3>
              <button className="text-secondary hover:text-primary transition-colors">
                <MoreHorizontal className="w-5 h-5" />
              </button>
            </div>
            
            <div className="flex-1 flex items-end justify-between gap-1 pt-4 border-b border-border/50 pb-1 px-1 bg-grid-pattern relative">
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[40%] transition-colors relative group">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">12</div>
              </div>
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[65%] transition-colors relative group">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">18</div>
              </div>
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[30%] transition-colors relative group">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">8</div>
              </div>
              <div className="w-full bg-primary rounded-t-sm h-[85%] transition-colors shadow-[0_0_8px_rgba(79,70,229,0.3)] relative group">
                 <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">24</div>
              </div>
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[50%] transition-colors relative group">
                 <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">15</div>
              </div>
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[20%] transition-colors relative group">
                 <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">5</div>
              </div>
              <div className="w-full bg-primary/20 hover:bg-primary/40 rounded-t-sm h-[70%] transition-colors relative group">
                 <div className="absolute -top-6 left-1/2 -translate-x-1/2 bg-inverse-surface text-inverse-on-surface font-mono text-[10px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">20</div>
              </div>
            </div>
            
            <div className="flex justify-between mt-1 font-mono text-[10px] text-muted-text px-1">
              <span>Mon</span><span>Tue</span><span>Wed</span><span>Thu</span><span>Fri</span><span>Sat</span><span>Sun</span>
            </div>
          </div>

          {/* Storage Summary */}
          <div className="bg-surface border border-border rounded-xl p-4 md:p-6 shadow-[0_1px_2px_rgba(0,0,0,0.05)] flex flex-col h-64">
            <h3 className="text-[11px] font-bold tracking-wide uppercase text-muted-text mb-4">Storage Utilization</h3>
            <div className="flex items-end gap-2 mb-2">
              <div className="text-3xl font-bold text-on-surface leading-tight">4.2 <span className="text-xl text-muted-text font-normal">TB</span></div>
              <div className="text-sm text-secondary pb-1">of 10 TB limit</div>
            </div>
            
            <div className="h-2 w-full bg-surface-container-high rounded-full overflow-hidden flex mb-4">
              <div className="h-full bg-primary" style={{ width: '30%' }}></div>
              <div className="h-full bg-surface-tint opacity-50" style={{ width: '12%' }}></div>
            </div>
            
            <div className="grid grid-cols-2 gap-2 mt-auto pt-4 border-t border-border/50">
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-primary"></span>
                <div>
                  <div className="text-[10px] font-bold text-muted-text uppercase tracking-wider">Run Outputs</div>
                  <div className="font-mono text-xs text-on-surface mt-0.5">3.0 TB</div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-surface-tint opacity-50"></span>
                <div>
                  <div className="text-[10px] font-bold text-muted-text uppercase tracking-wider">Manual Uploads</div>
                  <div className="font-mono text-xs text-on-surface mt-0.5">1.2 TB</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* ROW 3 */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Recent Executions Table */}
          <div className="lg:col-span-8 bg-surface border border-border rounded-xl shadow-[0_1px_2px_rgba(0,0,0,0.05)] overflow-hidden flex flex-col">
            <div className="p-4 border-b border-border flex items-center justify-between bg-surface/50 backdrop-blur-sm">
              <h3 className="text-[11px] font-bold tracking-wide uppercase text-muted-text">Recent Script Executions</h3>
              <button className="text-sm text-primary hover:underline font-medium">View All</button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse whitespace-nowrap">
                <thead>
                  <tr className="border-b border-border bg-surface-container-lowest">
                    <th className="text-[11px] font-bold tracking-wide uppercase text-muted-text py-2 px-4 w-32">Run ID</th>
                    <th className="text-[11px] font-bold tracking-wide uppercase text-muted-text py-2 px-4">Script</th>
                    <th className="text-[11px] font-bold tracking-wide uppercase text-muted-text py-2 px-4 w-24">Status</th>
                    <th className="text-[11px] font-bold tracking-wide uppercase text-muted-text py-2 px-4 w-24">Duration</th>
                    <th className="text-[11px] font-bold tracking-wide uppercase text-muted-text py-2 px-4 w-32 text-right">Started</th>
                  </tr>
                </thead>
                <tbody className="font-mono text-[13px] text-on-surface divide-y divide-border">
                  <tr className="hover:bg-surface-container-low transition-colors group">
                    <td className="py-2.5 px-4 text-primary cursor-pointer group-hover:underline">r-8f92a1b</td>
                    <td className="py-2.5 px-4 flex items-center gap-2">
                       <FileText className="w-3.5 h-3.5 text-secondary" />
                       analyze_genomics.py
                    </td>
                    <td className="py-2.5 px-4">
                       <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium bg-success/10 text-success border border-success/20 font-sans">
                         <CheckCircle2 className="w-3 h-3" /> Success
                       </span>
                    </td>
                    <td className="py-2.5 px-4 text-secondary">4m 12s</td>
                    <td className="py-2.5 px-4 text-secondary text-right">10 mins ago</td>
                  </tr>
                  
                  <tr className="hover:bg-surface-container-low transition-colors group">
                    <td className="py-2.5 px-4 text-primary cursor-pointer group-hover:underline">r-3c44f9d</td>
                    <td className="py-2.5 px-4 flex items-center gap-2">
                       <FileText className="w-3.5 h-3.5 text-secondary" />
                       clean_dataset_v2.py
                    </td>
                    <td className="py-2.5 px-4">
                       <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium bg-surface-container-high text-secondary border border-outline-variant font-sans">
                         <RefreshCw className="w-3 h-3 animate-spin" /> Running
                       </span>
                    </td>
                    <td className="py-2.5 px-4 text-secondary">12m 05s</td>
                    <td className="py-2.5 px-4 text-secondary text-right">12 mins ago</td>
                  </tr>

                  <tr className="hover:bg-surface-container-low transition-colors group">
                    <td className="py-2.5 px-4 text-primary cursor-pointer group-hover:underline">r-1a2b3c4</td>
                    <td className="py-2.5 px-4 flex items-center gap-2">
                       <FileText className="w-3.5 h-3.5 text-secondary" />
                       train_model_batch.py
                    </td>
                    <td className="py-2.5 px-4">
                       <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium bg-error-container/50 text-error border border-error/20 font-sans">
                         <AlertCircle className="w-3 h-3" /> Failed
                       </span>
                    </td>
                    <td className="py-2.5 px-4 text-secondary">1h 45m</td>
                    <td className="py-2.5 px-4 text-secondary text-right">2 hrs ago</td>
                  </tr>

                  <tr className="hover:bg-surface-container-low transition-colors group">
                    <td className="py-2.5 px-4 text-primary cursor-pointer group-hover:underline">r-9e8d7f6</td>
                    <td className="py-2.5 px-4 flex items-center gap-2">
                       <FileText className="w-3.5 h-3.5 text-secondary" />
                       export_results.py
                    </td>
                    <td className="py-2.5 px-4">
                       <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium bg-success/10 text-success border border-success/20 font-sans">
                         <CheckCircle2 className="w-3 h-3" /> Success
                       </span>
                    </td>
                    <td className="py-2.5 px-4 text-secondary">0m 45s</td>
                    <td className="py-2.5 px-4 text-secondary text-right">Yesterday</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          {/* Activity Feed */}
          <div className="lg:col-span-4 bg-surface border border-border rounded-xl shadow-[0_1px_2px_rgba(0,0,0,0.05)] overflow-hidden flex flex-col">
            <div className="p-4 border-b border-border bg-surface/50 backdrop-blur-sm">
              <h3 className="text-[11px] font-bold tracking-wide uppercase text-muted-text">Activity Feed</h3>
            </div>
            <div className="p-4 flex-1 overflow-y-auto">
              <div className="relative border-l border-border/50 ml-3 space-y-6 pb-2">
                
                <div className="relative pl-6">
                  <div className="absolute -left-1.5 top-1 w-3 h-3 rounded-full bg-surface border-2 border-primary"></div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold text-on-surface">System</span>
                    <span className="font-mono text-[10px] text-muted-text">10m ago</span>
                  </div>
                  <p className="text-sm text-secondary">Completed run <a href="#" className="font-mono text-primary hover:underline">r-8f92a1b</a> successfully. Output generated 142MB.</p>
                </div>

                <div className="relative pl-6">
                  <div className="absolute -left-1.5 top-1 w-3 h-3 rounded-full bg-surface border-2 border-secondary"></div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold text-on-surface">Dr. Sarah Chen</span>
                    <span className="font-mono text-[10px] text-muted-text">1h ago</span>
                  </div>
                  <p className="text-sm text-secondary">Uploaded new dataset <span className="font-mono text-on-surface bg-surface-container-low px-1 rounded border border-border">samples_v4.csv</span> (1.2GB).</p>
                </div>

                <div className="relative pl-6">
                  <div className="absolute -left-1.5 top-1 w-3 h-3 rounded-full bg-surface border-2 border-error"></div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold text-on-surface">System</span>
                    <span className="font-mono text-[10px] text-muted-text">2h ago</span>
                  </div>
                  <p className="text-sm text-secondary">Run <a href="#" className="font-mono text-error hover:underline">r-1a2b3c4</a> failed due to MemoryError in processing step 3.</p>
                </div>

                <div className="relative pl-6">
                  <div className="absolute -left-1.5 top-1 w-3 h-3 rounded-full bg-surface border-2 border-secondary"></div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold text-on-surface">You</span>
                    <span className="font-mono text-[10px] text-muted-text">Yesterday</span>
                  </div>
                  <p className="text-sm text-secondary">Modified configuration file <span className="font-mono text-on-surface bg-surface-container-low px-1 rounded border border-border">config.yaml</span>.</p>
                </div>

              </div>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
