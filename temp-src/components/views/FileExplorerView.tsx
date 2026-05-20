import { Folder, FolderPlus, Upload, Plus, FileText, FileSpreadsheet, FileCode, Image as ImageIcon, Eye, Download, Trash2, ChevronRight, FolderOpen } from "lucide-react";

export function FileExplorerView() {
  return (
    <div className="flex-1 flex overflow-hidden">
      {/* Directory Tree Pane */}
      <aside className="w-64 border-r border-outline-variant bg-surface flex flex-col shrink-0 hidden lg:flex">
        <div className="p-3 border-b border-outline-variant flex items-center justify-between">
          <span className="text-[11px] font-bold tracking-wider text-secondary uppercase">Directories</span>
          <button className="w-6 h-6 flex items-center justify-center rounded text-secondary hover:bg-surface-container-low hover:text-on-surface transition-colors">
            <FolderPlus className="w-4 h-4" />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-2">
          <ul className="space-y-[2px]">
            <li>
              <button className="w-full flex items-center gap-2 px-2 py-1 rounded text-on-surface hover:bg-surface-container-low transition-colors text-left text-sm group">
                <Folder className="w-4 h-4 text-secondary group-hover:text-on-surface" fill="currentColor" fillOpacity={0.2} />
                <span>uploads/</span>
              </button>
            </li>
            
            <li>
              <div className="w-full flex flex-col">
                <button className="w-full flex items-center gap-2 px-2 py-1 rounded bg-surface-container-low text-on-surface transition-colors text-left text-sm font-medium">
                  <FolderOpen className="w-4 h-4 text-primary" fill="currentColor" fillOpacity={0.2} />
                  <span>workspace/</span>
                </button>
                <ul className="ml-4 border-l border-outline-variant pl-2 mt-[2px] space-y-[2px]">
                  <li>
                    <button className="w-full flex items-center gap-2 px-2 py-1 rounded text-on-surface hover:bg-surface-container-low transition-colors text-left text-sm group">
                      <Folder className="w-4 h-4 text-secondary group-hover:text-on-surface" fill="currentColor" fillOpacity={0.2} />
                      <span>data_raw/</span>
                    </button>
                  </li>
                  <li>
                    <button className="w-full flex items-center gap-2 px-2 py-1 rounded text-on-surface hover:bg-surface-container-low transition-colors text-left text-sm group">
                      <Folder className="w-4 h-4 text-secondary group-hover:text-on-surface" fill="currentColor" fillOpacity={0.2} />
                      <span>scripts/</span>
                    </button>
                  </li>
                </ul>
              </div>
            </li>

            <li>
              <button className="w-full flex items-center gap-2 px-2 py-1 rounded text-on-surface hover:bg-surface-container-low transition-colors text-left text-sm group">
                <Folder className="w-4 h-4 text-secondary group-hover:text-on-surface" fill="currentColor" fillOpacity={0.2} />
                <span>runs/</span>
              </button>
            </li>

            <li className="mt-4">
              <button className="w-full flex items-center gap-2 px-2 py-1 rounded text-secondary hover:bg-surface-container-low hover:text-on-surface transition-colors text-left text-sm group">
                <Trash2 className="w-4 h-4" />
                <span>trash/</span>
              </button>
            </li>
          </ul>
        </div>
      </aside>

      {/* File Table Pane */}
      <main className="flex-1 flex flex-col min-w-0 bg-background">
        
        {/* Toolbar */}
        <div className="h-12 border-b border-outline-variant bg-surface flex items-center justify-between px-3 shrink-0">
          <div className="flex items-center text-sm font-medium text-secondary">
            <span className="hover:text-on-surface cursor-pointer">workspace</span>
            <ChevronRight className="w-4 h-4 mx-1" />
            <span className="text-on-surface">scripts</span>
          </div>
          
          <div className="flex items-center gap-2">
            <button className="h-8 px-3 rounded border border-outline-variant bg-surface text-secondary hover:bg-surface-container-low hover:text-on-surface transition-colors flex items-center gap-1.5 text-sm font-medium">
              <Upload className="w-4 h-4" /> Upload
            </button>
            <button className="h-8 px-3 rounded bg-primary text-on-primary hover:bg-primary-fixed-variant transition-colors flex items-center gap-1.5 text-sm font-medium shadow-sm">
              <Plus className="w-4 h-4" /> New File
            </button>
          </div>
        </div>

        {/* Table Container */}
        <div className="flex-1 overflow-auto bg-surface relative">
          <table className="w-full text-left border-collapse min-w-[600px]">
            <thead className="sticky top-0 bg-surface-container-lowest z-10 shadow-[0_1px_2px_rgba(0,0,0,0.05)]">
              <tr className="border-b border-outline-variant text-[11px] font-bold text-secondary uppercase tracking-wider">
                <th className="w-10 px-4 py-2 text-center">
                  <input type="checkbox" className="rounded-[2px] border-outline text-primary focus:ring-primary w-3.5 h-3.5 cursor-pointer bg-transparent" />
                </th>
                <th className="px-4 py-2 font-bold">Name</th>
                <th className="w-32 px-4 py-2 font-bold">Type</th>
                <th className="w-24 px-4 py-2 font-bold text-right">Size</th>
                <th className="w-32 px-4 py-2 font-bold">Modified</th>
                <th className="w-24 px-4 py-2 font-bold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="text-[13px] text-on-surface divide-y divide-outline-variant">
              {/* Row 1 */}
              <tr className="hover:bg-surface-container-low transition-colors group">
                <td className="px-4 py-2 text-center">
                  <input type="checkbox" className="rounded-[2px] border-outline text-primary focus:ring-primary w-3.5 h-3.5 cursor-pointer bg-transparent" />
                </td>
                <td className="px-4 py-2 font-medium flex items-center gap-2">
                  <FileText className="w-4 h-4 text-secondary flex-shrink-0" />
                  <span className="truncate">data_processor.py</span>
                </td>
                <td className="px-4 py-2 text-secondary">Python File</td>
                <td className="px-4 py-2 text-right font-mono text-secondary">14.2 KB</td>
                <td className="px-4 py-2 text-secondary whitespace-nowrap">10 mins ago</td>
                <td className="px-4 py-2">
                  <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Preview"><Eye className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Download"><Download className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-destructive hover:bg-error-container flex items-center justify-center transition-colors" title="Delete"><Trash2 className="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>

              {/* Row 2 */}
              <tr className="hover:bg-surface-container-low transition-colors group">
                <td className="px-4 py-2 text-center">
                  <input type="checkbox" className="rounded-[2px] border-outline text-primary focus:ring-primary w-3.5 h-3.5 cursor-pointer bg-transparent" />
                </td>
                <td className="px-4 py-2 font-medium flex items-center gap-2">
                  <FileSpreadsheet className="w-4 h-4 text-success flex-shrink-0" />
                  <span className="truncate">results_matrix_v2.csv</span>
                </td>
                <td className="px-4 py-2 text-secondary">CSV Document</td>
                <td className="px-4 py-2 text-right font-mono text-secondary">2.1 MB</td>
                <td className="px-4 py-2 text-secondary whitespace-nowrap">2 hours ago</td>
                <td className="px-4 py-2">
                  <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Preview"><Eye className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Download"><Download className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-destructive hover:bg-error-container flex items-center justify-center transition-colors" title="Delete"><Trash2 className="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>

              {/* Row 3 Selected */}
              <tr className="bg-surface-container-low group">
                <td className="px-4 py-2 text-center">
                  <input type="checkbox" defaultChecked className="rounded-[2px] border-primary text-primary focus:ring-primary w-3.5 h-3.5 cursor-pointer" />
                </td>
                <td className="px-4 py-2 font-medium flex items-center gap-2 text-primary">
                  <FileCode className="w-4 h-4 text-primary flex-shrink-0" />
                  <span className="truncate">config_defaults.json</span>
                </td>
                <td className="px-4 py-2 text-secondary">JSON File</td>
                <td className="px-4 py-2 text-right font-mono text-secondary">842 B</td>
                <td className="px-4 py-2 text-secondary whitespace-nowrap">Yesterday</td>
                <td className="px-4 py-2">
                  <div className="flex justify-end gap-1 opacity-100 transition-opacity">
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Preview"><Eye className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Download"><Download className="w-4 h-4" /></button>
                    <button className="w-6 h-6 rounded text-secondary hover:text-destructive hover:bg-error-container flex items-center justify-center transition-colors" title="Delete"><Trash2 className="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>

              {/* Row 4 */}
              <tr className="hover:bg-surface-container-low transition-colors group">
                <td className="px-4 py-2 text-center">
                  <input type="checkbox" className="rounded-[2px] border-outline text-primary focus:ring-primary w-3.5 h-3.5 cursor-pointer bg-transparent" />
                </td>
                <td className="px-4 py-2 font-medium flex items-center gap-2">
                  <ImageIcon className="w-4 h-4 text-warning flex-shrink-0" />
                  <span className="truncate">plot_output_final.png</span>
                </td>
                <td className="px-4 py-2 text-secondary">Image</td>
                <td className="px-4 py-2 text-right font-mono text-secondary">450 KB</td>
                <td className="px-4 py-2 text-secondary whitespace-nowrap">Yesterday</td>
                <td className="px-4 py-2">
                  <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                     <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Preview"><Eye className="w-4 h-4" /></button>
                     <button className="w-6 h-6 rounded text-secondary hover:text-primary hover:bg-surface-variant flex items-center justify-center transition-colors" title="Download"><Download className="w-4 h-4" /></button>
                     <button className="w-6 h-6 rounded text-secondary hover:text-destructive hover:bg-error-container flex items-center justify-center transition-colors" title="Delete"><Trash2 className="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>

            </tbody>
          </table>
        </div>

        {/* Status Bar */}
        <div className="h-8 bg-surface-container-lowest border-t border-outline-variant flex items-center px-4 text-[11px] font-bold tracking-wider uppercase text-secondary shrink-0">
          <span>1 item selected</span>
          <span className="mx-2">•</span>
          <span>Total size: 842 B</span>
        </div>

      </main>
    </div>
  );
}
