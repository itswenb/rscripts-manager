import { ChevronDown, Database, UploadCloud, X, Rocket, Info, CheckCircle2 } from "lucide-react";

export function ConfigureRunView() {
  return (
    <div className="flex-1 overflow-y-auto p-4 md:p-6 bg-background">
      <div className="max-w-[1280px] mx-auto flex flex-col xl:flex-row gap-6 relative items-start">
        
        {/* Left Column: Form Area */}
        <div className="flex-1 flex flex-col gap-6 min-w-0 w-full">
          {/* Page Header */}
          <div>
            <h1 className="text-[30px] font-bold text-on-surface mb-1">Configure Run</h1>
            <p className="text-sm text-on-surface-variant">Define workflow script, map required inputs, and set execution parameters.</p>
          </div>

          {/* Step 1: Select Workflow */}
          <section className="bg-surface-container-lowest border border-border rounded-xl flex flex-col shadow-[0_1px_2px_rgba(0,0,0,0.05)] overflow-hidden">
            <div className="bg-surface-container-low px-4 md:px-6 py-4 border-b border-border flex items-center gap-3">
               <div className="w-6 h-6 rounded-full bg-primary text-on-primary flex items-center justify-center text-[11px] font-bold">1</div>
               <h3 className="text-lg font-bold text-on-surface">Select Workflow Script</h3>
            </div>
            <div className="p-4 md:p-6">
               <label className="block text-[11px] font-bold text-on-surface-variant mb-2 uppercase tracking-wide">Approved Scripts Repository</label>
               <div className="relative">
                 <select defaultValue="variant_calling" className="w-full appearance-none bg-surface border border-outline text-on-surface text-sm rounded px-3 py-2.5 pr-10 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors cursor-pointer">
                    <option disabled value="">Select a script to configure...</option>
                    <option value="rnaseq_qc">RNA-Seq Quality Control Pipeline (v2.1)</option>
                    <option value="variant_calling">GATK Variant Calling Workflow (v4.3)</option>
                    <option value="metagenomics">Metagenomic Assembly &amp; Annotation</option>
                 </select>
                 <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 text-outline w-5 h-5 pointer-events-none" />
               </div>
               
               <div className="mt-4 bg-surface-container p-3 rounded border border-outline-variant flex flex-col gap-1">
                  <div className="flex items-center justify-between">
                     <span className="font-mono text-xs text-secondary">gatk_variant_calling.nf</span>
                     <span className="px-2 py-0.5 bg-success/20 text-success rounded text-[10px] font-bold uppercase tracking-wider">Validated</span>
                  </div>
                  <p className="text-[13px] text-on-surface-variant mt-1 leading-relaxed">
                     Standard pipeline for identifying SNPs and indels in germline DNA sequencing data using Genome Analysis Toolkit.
                  </p>
               </div>
            </div>
          </section>

          {/* Step 2: Map Inputs */}
          <section className="bg-surface-container-lowest border border-border rounded-xl flex flex-col shadow-[0_1px_2px_rgba(0,0,0,0.05)] overflow-hidden">
             <div className="bg-surface-container-low px-4 md:px-6 py-4 border-b border-border flex items-center gap-3">
               <div className="w-6 h-6 rounded-full bg-primary text-on-primary flex items-center justify-center text-[11px] font-bold">2</div>
               <h3 className="text-lg font-bold text-on-surface">Map Inputs</h3>
             </div>
             <div className="p-4 md:p-6 flex flex-col gap-6">
                
                {/* Input 1 */}
                <div className="flex flex-col gap-2">
                   <div className="flex items-center justify-between">
                      <label className="text-[11px] font-bold text-on-surface uppercase tracking-wide">
                        Reference Genome <span className="text-destructive">*</span>
                      </label>
                      <span className="font-mono text-[10px] text-secondary">.fasta, .fa</span>
                   </div>
                   <div className="flex gap-2 sm:flex-row flex-col">
                      <div className="flex-1 relative flex items-center border border-outline rounded bg-surface overflow-hidden focus-within:border-primary focus-within:ring-1 focus-within:ring-primary transition-all pr-2">
                         <div className="pl-3 pr-2 text-outline flex items-center"><Database className="w-4 h-4" /></div>
                         <input type="text" readOnly className="w-full bg-transparent border-none font-mono text-[13px] text-on-surface focus:ring-0 py-2.5 outline-none" value="s3://project-alpha/references/hg38.fa" />
                      </div>
                      <button className="px-4 py-2 border border-outline text-secondary hover:bg-surface-container-low hover:text-on-surface rounded text-sm transition-colors whitespace-nowrap font-medium">
                          Browse
                      </button>
                   </div>
                </div>

                <hr className="border-border" />

                {/* Input 2 */}
                <div className="flex flex-col gap-2">
                   <div className="flex items-center justify-between">
                      <label className="text-[11px] font-bold text-on-surface uppercase tracking-wide">
                        Input BAM Files <span className="text-destructive">*</span>
                      </label>
                      <span className="font-mono text-[10px] text-secondary">.bam</span>
                   </div>
                   
                   <div className="border border-dashed border-outline-variant bg-surface hover:bg-surface-container-low transition-colors rounded flex flex-col items-center justify-center p-6 cursor-pointer group">
                      <UploadCloud className="w-8 h-8 text-outline group-hover:text-primary transition-colors mb-2" />
                      <p className="text-[13px] text-on-surface text-center">Click to select files or drag and drag here</p>
                      <p className="text-[11px] text-secondary text-center mt-1">Supports multiple .bam files</p>
                   </div>

                   <div className="flex flex-col gap-2 mt-1">
                      <div className="flex items-center justify-between p-2 pl-3 bg-surface border border-outline-variant rounded">
                         <div className="flex items-center gap-2">
                            <span className="text-secondary"><UploadCloud className="w-4 h-4" /></span>
                            <span className="font-mono text-[13px] text-on-surface truncate">sample_01_tumor.bam</span>
                         </div>
                         <button className="text-secondary hover:text-destructive p-1 rounded hover:bg-error-container transition-colors"><X className="w-4 h-4" /></button>
                      </div>
                      <div className="flex items-center justify-between p-2 pl-3 bg-surface border border-outline-variant rounded">
                         <div className="flex items-center gap-2">
                            <span className="text-secondary"><UploadCloud className="w-4 h-4" /></span>
                            <span className="font-mono text-[13px] text-on-surface truncate">sample_02_normal.bam</span>
                         </div>
                         <button className="text-secondary hover:text-destructive p-1 rounded hover:bg-error-container transition-colors"><X className="w-4 h-4" /></button>
                      </div>
                   </div>
                </div>

             </div>
          </section>

          {/* Step 3: Set Parameters */}
          <section className="bg-surface-container-lowest border border-border rounded-xl flex flex-col shadow-[0_1px_2px_rgba(0,0,0,0.05)] overflow-hidden">
             <div className="bg-surface-container-low px-4 md:px-6 py-4 border-b border-border flex items-center justify-between">
                <div className="flex items-center gap-3">
                   <div className="w-6 h-6 rounded-full bg-primary text-on-primary flex items-center justify-center text-[11px] font-bold">3</div>
                   <h3 className="text-lg font-bold text-on-surface">Execution Parameters</h3>
                </div>
                <button className="text-[13px] text-primary hover:underline font-medium">Load Defaults</button>
             </div>
             
             <div className="p-4 md:p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                   <div className="flex flex-col gap-2">
                      <label className="text-[11px] font-bold text-on-surface-variant uppercase tracking-wide">CPU Threads</label>
                      <input type="number" defaultValue="16" className="bg-surface border border-outline text-on-surface font-mono text-[13px] rounded px-3 py-2.5 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors outline-none" />
                   </div>
                   <div className="flex flex-col gap-2">
                      <label className="text-[11px] font-bold text-on-surface-variant uppercase tracking-wide">Memory Allocation (GB)</label>
                      <input type="number" defaultValue="64" className="bg-surface border border-outline text-on-surface font-mono text-[13px] rounded px-3 py-2.5 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors outline-none" />
                   </div>
                   <div className="flex flex-col gap-2 md:col-span-2">
                      <label className="text-[11px] font-bold text-on-surface-variant uppercase tracking-wide">Calling Mode</label>
                      <div className="flex gap-3 flex-col sm:flex-row">
                         <label className="flex-1 border border-primary bg-primary-container/10 rounded p-3 cursor-pointer flex items-center justify-center gap-2 transition-colors">
                            <input type="radio" name="call_mode" defaultChecked className="text-primary focus:ring-primary w-4 h-4" />
                            <span className="text-[13px] font-bold text-primary">Discovery</span>
                         </label>
                         <label className="flex-1 border border-outline bg-surface rounded p-3 cursor-pointer flex items-center justify-center gap-2 hover:bg-surface-container-low transition-colors">
                            <input type="radio" name="call_mode" className="text-primary focus:ring-primary w-4 h-4 cursor-pointer" />
                            <span className="text-[13px] text-on-surface">Genotype Given Alleles</span>
                         </label>
                      </div>
                   </div>

                   {/* Toggle */}
                   <div className="flex items-center justify-between md:col-span-2 py-4 border-t border-border mt-2">
                      <div className="pr-4">
                         <h4 className="text-sm font-semibold text-on-surface">Enable Base Quality Score Recalibration (BQSR)</h4>
                         <p className="text-[13px] text-secondary mt-0.5">Recommended for accurate variant calling.</p>
                      </div>
                      <button className="w-10 h-6 bg-primary rounded-full relative flex-shrink-0 transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2">
                         <span className="absolute right-1 top-1 w-4 h-4 bg-white rounded-full shadow transition-transform"></span>
                      </button>
                   </div>
                </div>
             </div>
          </section>

        </div>

        {/* Right Column: Summary */}
        <aside className="w-full xl:w-[320px] flex-shrink-0 xl:sticky xl:top-6">
           <div className="bg-surface-container-lowest border border-border rounded-xl shadow-[0_1px_2px_rgba(0,0,0,0.05)] flex flex-col">
              <div className="p-4 border-b border-border bg-surface-bright rounded-t-xl">
                 <h3 className="text-lg font-bold text-on-surface mb-1">Run Summary</h3>
                 <div className="font-mono text-xs text-secondary flex items-center gap-1.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-warning"></span>
                    Draft Configuration
                 </div>
              </div>

              <div className="p-2 flex flex-col gap-[1px] bg-border">
                 <div className="bg-surface-container-lowest p-3 flex flex-col gap-1">
                    <span className="text-[11px] font-bold text-secondary uppercase tracking-wider">Workflow</span>
                    <span className="text-[13px] font-semibold text-on-surface truncate">GATK Variant Calling (v4.3)</span>
                 </div>
                 <div className="bg-surface-container-lowest p-3 flex flex-col gap-1">
                    <span className="text-[11px] font-bold text-secondary uppercase tracking-wider mb-1 mt-1">Inputs Mapped</span>
                    <div className="flex justify-between items-center py-0.5">
                       <span className="font-mono text-[13px] text-on-surface">Reference</span>
                       <CheckCircle2 className="w-4 h-4 text-success" />
                    </div>
                    <div className="flex justify-between items-center py-0.5">
                       <span className="font-mono text-[13px] text-on-surface">BAM Files (2)</span>
                       <CheckCircle2 className="w-4 h-4 text-success" />
                    </div>
                 </div>
                 <div className="bg-surface-container-lowest p-3 flex flex-col gap-1">
                    <span className="text-[11px] font-bold text-secondary uppercase tracking-wider mb-1 mt-1">Compute Estimate</span>
                    <div className="grid grid-cols-2 gap-2 mt-1">
                       <div className="bg-surface-container p-2 rounded border border-outline-variant">
                          <div className="font-mono text-[10px] text-secondary">CPUs</div>
                          <div className="font-mono text-[13px] text-on-surface mt-0.5">16</div>
                       </div>
                       <div className="bg-surface-container p-2 rounded border border-outline-variant">
                          <div className="font-mono text-[10px] text-secondary">RAM</div>
                          <div className="font-mono text-[13px] text-on-surface mt-0.5">64 GB</div>
                       </div>
                    </div>
                    <div className="mt-3 flex items-start gap-1.5 text-secondary">
                       <Info className="w-4 h-4 flex-shrink-0 mt-0.5" />
                       <span className="text-[11px] leading-tight">Estimated cost based on current queue: ~$4.20 / hour.</span>
                    </div>
                 </div>
              </div>

              <div className="p-4 bg-surface-container-lowest rounded-b-xl border-t border-border mt-[1px]">
                 <button className="w-full bg-primary hover:bg-primary-fixed-variant text-on-primary text-[15px] font-bold py-2.5 px-4 rounded-lg shadow-sm transition-all active:scale-[0.98] flex items-center justify-center gap-2">
                    <Rocket className="w-5 h-5 fill-current" /> Launch Run
                 </button>
                 <button className="w-full mt-3 bg-transparent border border-outline hover:bg-surface-container-low text-secondary text-sm font-semibold py-2.5 px-4 rounded-lg transition-colors">
                    Save as Draft
                 </button>
              </div>
           </div>
        </aside>

      </div>
    </div>
  );
}
