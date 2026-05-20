import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useProjects } from "@/lib/queries/projects";
import { usePipelines, useCreatePipeline, useDeletePipeline, useStartPipelineRun, usePipelineRuns, usePipelineRunDetail, useStepOutputs, usePipeline } from "@/lib/queries/pipelines";
import { useScripts, ScriptInfo } from "@/lib/queries/scripts";
import { useMyFiles, UserFile } from "@/lib/queries/user-files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Plus, Play, Trash2, MoreHorizontal, ArrowRight, FileCode, ChevronDown, Image, FileText, Check } from "lucide-react";
import type { Pipeline, PipelineStepRun, StepOutputFile } from "@/lib/queries/pipelines";

export function PipelinesPage() {
  const { t } = useTranslation();
  const { data: projects } = useProjects();
  const [selectedProject, setSelectedProject] = useState<string>("");
  const projectId = selectedProject || projects?.[0]?.id || "";
  const { data: pipelines, isLoading } = usePipelines(projectId);
  const [createOpen, setCreateOpen] = useState(false);
  const [viewRunId, setViewRunId] = useState<{ pipelineId: string; runId: string } | null>(null);

  if (!projects?.length) {
    return <p className="p-6 text-sm text-muted-foreground">{t("common.loading")}</p>;
  }

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">{t("pipelines.title")}</h1>
          <Select value={projectId} onValueChange={setSelectedProject}>
            <SelectTrigger className="w-48 h-8 text-sm">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {projects.map((p) => (
                <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <Plus size={14} className="mr-1.5" />
          {t("pipelines.create")}
        </Button>
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">{t("common.loading")}</p>}

      {pipelines?.length === 0 && !isLoading && (
        <p className="text-sm text-muted-foreground py-8 text-center">{t("pipelines.noPipelines")}</p>
      )}

      <div className="space-y-3">
        {pipelines?.map((p) => (
          <PipelineCard
            key={p.id}
            pipeline={p}
            projectId={projectId}
            onViewRun={(runId) => setViewRunId({ pipelineId: p.id, runId })}
          />
        ))}
      </div>

      <CreatePipelineDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        projectId={projectId}
      />

      {viewRunId && (
        <RunDetailDialog
          open={!!viewRunId}
          onOpenChange={() => setViewRunId(null)}
          projectId={projectId}
          pipelineId={viewRunId.pipelineId}
          runId={viewRunId.runId}
        />
      )}
    </div>
  );
}

function PipelineCard({ pipeline, projectId, onViewRun }: { pipeline: Pipeline; projectId: string; onViewRun: (runId: string) => void }) {
  const { t } = useTranslation();
  const deletePipeline = useDeletePipeline(projectId);
  const { data: runs } = usePipelineRuns(projectId, pipeline.id);
  const [runDialogOpen, setRunDialogOpen] = useState(false);

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">{pipeline.name}</CardTitle>
          <div className="flex items-center gap-1">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setRunDialogOpen(true)}
            >
              <Play size={12} className="mr-1" />
              {t("pipelines.run")}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground">
                  <MoreHorizontal size={14} />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  className="text-destructive focus:text-destructive"
                  onClick={() => deletePipeline.mutate(pipeline.id)}
                >
                  <Trash2 size={13} className="mr-2" />
                  {t("common.delete")}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        {pipeline.description && (
          <p className="text-xs text-muted-foreground">{pipeline.description}</p>
        )}
      </CardHeader>
      {runs && runs.length > 0 && (
        <CardContent className="pt-0">
          <div className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground">{t("pipelines.runs")}</p>
            {runs.slice(0, 3).map((run) => (
              <div key={run.id} className="flex items-center justify-between text-xs py-1">
                <div className="flex items-center gap-2">
                  <StatusBadge status={run.status} />
                  <span className="text-muted-foreground font-mono">{run.id.slice(0, 8)}</span>
                </div>
                <Button variant="ghost" size="sm" className="h-6 text-xs" onClick={() => onViewRun(run.id)}>
                  {t("pipelines.viewRun")}
                </Button>
              </div>
            ))}
          </div>
        </CardContent>
      )}
      <StartRunDialog
        open={runDialogOpen}
        onOpenChange={setRunDialogOpen}
        projectId={projectId}
        pipelineId={pipeline.id}
      />
    </Card>
  );
}

function StatusBadge({ status }: { status: string }) {
  const variant = status === "completed" ? "default" : status === "failed" ? "destructive" : "secondary";
  return <Badge variant={variant} className="text-[10px] px-1.5 py-0">{status}</Badge>;
}

function StartRunDialog({ open, onOpenChange, projectId, pipelineId }: { open: boolean; onOpenChange: (v: boolean) => void; projectId: string; pipelineId: string }) {
  const { t } = useTranslation();
  const startRun = useStartPipelineRun(projectId, pipelineId);
  const { data: pipelineDetail } = usePipeline(projectId, pipelineId);
  const { data: scripts } = useScripts();
  const { data: files } = useMyFiles();
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [paramValues, setParamValues] = useState<Record<string, string>>({});

  // Get all params from the first step's script
  const firstStepScript = pipelineDetail?.steps?.[0]?.script_path;
  const scriptMeta = scripts?.find((s) => s.storage_path === firstStepScript);
  const params = scriptMeta?.meta.params || [];
  const expectedInputs = scriptMeta?.meta.inputs || [];

  function toggleFile(fileId: string) {
    setSelectedFiles((prev) =>
      prev.includes(fileId) ? prev.filter((id) => id !== fileId) : [...prev, fileId]
    );
  }

  async function handleStart() {
    const param_overrides: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(paramValues)) {
      if (v.trim()) param_overrides[k] = v.trim();
    }
    await startRun.mutateAsync({
      input_files: selectedFiles.length > 0 ? selectedFiles : undefined,
      param_overrides: Object.keys(param_overrides).length > 0 ? param_overrides : undefined,
    });
    setSelectedFiles([]);
    setParamValues({});
    onOpenChange(false);
  }

  const dataFiles = files?.filter((f) => !f.is_directory) || [];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("pipelines.run")}</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 pt-2">
          {expectedInputs.length > 0 && (
            <div className="space-y-2">
              <Label>{t("pipelines.inputs")}</Label>
              <p className="text-xs text-muted-foreground">
                {expectedInputs.map((i) => i.name).join(", ")}
              </p>
              <div className="border rounded max-h-40 overflow-y-auto">
                {dataFiles.length === 0 && (
                  <p className="text-xs text-muted-foreground p-2">{t("files.noFiles")}</p>
                )}
                {dataFiles.map((f) => (
                  <button
                    key={f.id}
                    type="button"
                    onClick={() => toggleFile(f.id)}
                    className="flex items-center gap-2 w-full text-left px-2 py-1.5 text-sm hover:bg-accent"
                  >
                    <div className={`w-4 h-4 border rounded flex items-center justify-center ${selectedFiles.includes(f.id) ? "bg-primary border-primary" : ""}`}>
                      {selectedFiles.includes(f.id) && <Check size={10} className="text-primary-foreground" />}
                    </div>
                    <FileText size={13} className="text-muted-foreground" />
                    <span className="flex-1 truncate">{f.name}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {params.length > 0 && (
            <div className="space-y-2">
              <Label>{t("pipelines.params")}</Label>
              {params.map((p) => (
                <div key={p.name} className="flex items-center gap-2">
                  <Label className="text-xs w-28 shrink-0">{p.name}</Label>
                  <Input
                    className="h-7 text-sm"
                    placeholder={p.default || ""}
                    value={paramValues[p.name] || ""}
                    onChange={(e) => setParamValues({ ...paramValues, [p.name]: e.target.value })}
                  />
                  <span className="text-[10px] text-muted-foreground shrink-0">{p.type}</span>
                </div>
              ))}
            </div>
          )}

          {expectedInputs.length === 0 && params.length === 0 && (
            <p className="text-sm text-muted-foreground">{t("pipelines.run")}?</p>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>{t("common.cancel")}</Button>
          <Button onClick={handleStart} disabled={startRun.isPending}>
            <Play size={12} className="mr-1" />
            {t("pipelines.run")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function CreatePipelineDialog({ open, onOpenChange, projectId }: { open: boolean; onOpenChange: (v: boolean) => void; projectId: string }) {
  const { t } = useTranslation();
  const { data: scripts } = useScripts();
  const createPipeline = useCreatePipeline(projectId);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [steps, setSteps] = useState<{ script_path: string; label: string }[]>([]);

  function addScript(script: ScriptInfo) {
    setSteps([...steps, { script_path: script.storage_path, label: script.meta.title || script.name }]);
  }

  function removeStep(idx: number) {
    setSteps(steps.filter((_, i) => i !== idx));
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || steps.length === 0) return;
    await createPipeline.mutateAsync({ name: name.trim(), description: description.trim() || undefined, steps });
    setName(""); setDescription(""); setSteps([]);
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("pipelines.create")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleCreate} className="space-y-4 pt-2">
          <div className="space-y-1.5">
            <Label>{t("pipelines.name")}</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus />
          </div>
          <div className="space-y-1.5">
            <Label>{t("common.description")}</Label>
            <Input value={description} onChange={(e) => setDescription(e.target.value)} />
          </div>

          <div className="space-y-2">
            <Label>{t("pipelines.steps")}</Label>
            {steps.length === 0 && (
              <p className="text-xs text-muted-foreground">{t("pipelines.addScript")}</p>
            )}
            {steps.map((s, i) => (
              <div key={i} className="flex items-center gap-2 text-sm bg-muted/50 rounded px-2 py-1.5">
                <span className="text-xs font-mono text-muted-foreground w-5">{i + 1}.</span>
                <FileCode size={13} className="text-muted-foreground shrink-0" />
                <span className="flex-1 truncate">{s.label}</span>
                {i < steps.length - 1 && <ArrowRight size={12} className="text-muted-foreground" />}
                <button type="button" onClick={() => removeStep(i)} className="text-muted-foreground hover:text-destructive">
                  <Trash2 size={12} />
                </button>
              </div>
            ))}
          </div>

          <div className="space-y-1.5">
            <Label>{t("pipelines.availableScripts")}</Label>
            {!scripts?.length && (
              <p className="text-xs text-muted-foreground">{t("pipelines.noScripts")}</p>
            )}
            <div className="grid gap-1 max-h-40 overflow-y-auto">
              {scripts?.map((s) => (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => addScript(s)}
                  className="flex items-center gap-2 text-left text-sm px-2 py-1.5 rounded hover:bg-accent"
                >
                  <FileCode size={13} className="text-muted-foreground shrink-0" />
                  <div className="flex-1 min-w-0">
                    <span className="font-medium">{s.meta.title || s.name}</span>
                    {s.meta.description && (
                      <span className="text-xs text-muted-foreground ml-2">{s.meta.description}</span>
                    )}
                  </div>
                  <Plus size={12} className="text-muted-foreground" />
                </button>
              ))}
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>{t("common.cancel")}</Button>
            <Button type="submit" disabled={createPipeline.isPending || !name.trim() || steps.length === 0}>{t("common.create")}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function RunDetailDialog({ open, onOpenChange, projectId, pipelineId, runId }: { open: boolean; onOpenChange: (v: boolean) => void; projectId: string; pipelineId: string; runId: string }) {
  const { t } = useTranslation();
  const { data: detail } = usePipelineRunDetail(projectId, pipelineId, runId);
  const [selectedStep, setSelectedStep] = useState<string | null>(null);

  if (!detail) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {t("pipelines.viewRun")}
            <StatusBadge status={detail.status} />
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          {detail.step_runs.map((sr) => (
            <StepRunCard
              key={sr.id}
              stepRun={sr}
              projectId={projectId}
              pipelineId={pipelineId}
              runId={runId}
              expanded={selectedStep === sr.id}
              onToggle={() => setSelectedStep(selectedStep === sr.id ? null : sr.id)}
            />
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function StepRunCard({ stepRun, projectId, pipelineId, runId, expanded, onToggle }: { stepRun: PipelineStepRun; projectId: string; pipelineId: string; runId: string; expanded: boolean; onToggle: () => void }) {
  const { t } = useTranslation();
  const { data: outputs } = useStepOutputs(projectId, pipelineId, runId, expanded ? stepRun.id : "");

  const scriptName = stepRun.script_path.split("/").pop() || stepRun.script_path;

  return (
    <Card>
      <button onClick={onToggle} className="w-full text-left px-4 py-2.5 flex items-center gap-2">
        <ChevronDown size={14} className={`transition-transform ${expanded ? "" : "-rotate-90"}`} />
        <span className="text-xs font-mono text-muted-foreground">#{stepRun.step_order + 1}</span>
        <span className="text-sm font-medium flex-1">{scriptName}</span>
        <StatusBadge status={stepRun.status} />
      </button>
      {expanded && (
        <CardContent className="pt-0 space-y-3">
          {stepRun.stdout && (
            <div>
              <p className="text-xs font-medium mb-1">{t("pipelines.stdout")}</p>
              <pre className="text-xs bg-muted rounded p-2 max-h-32 overflow-auto whitespace-pre-wrap">{stepRun.stdout}</pre>
            </div>
          )}
          {stepRun.stderr && (
            <div>
              <p className="text-xs font-medium mb-1 text-destructive">{t("pipelines.stderr")}</p>
              <pre className="text-xs bg-destructive/10 rounded p-2 max-h-32 overflow-auto whitespace-pre-wrap">{stepRun.stderr}</pre>
            </div>
          )}
          {outputs && outputs.length > 0 && (
            <div>
              <p className="text-xs font-medium mb-1">{t("pipelines.stepOutputs")}</p>
              <div className="grid gap-2">
                {outputs.map((o) => (
                  <OutputFilePreview key={o.id} file={o} />
                ))}
              </div>
            </div>
          )}
          {outputs?.length === 0 && stepRun.status === "completed" && (
            <p className="text-xs text-muted-foreground">{t("pipelines.noOutputs")}</p>
          )}
        </CardContent>
      )}
    </Card>
  );
}

function OutputFilePreview({ file }: { file: StepOutputFile }) {
  const isImage = file.mime_type?.startsWith("image/");
  const downloadUrl = `/api/step-outputs/${file.id}/download`;

  return (
    <div className="border rounded p-2">
      <div className="flex items-center gap-2 mb-1">
        {isImage ? <Image size={13} className="text-muted-foreground" /> : <FileText size={13} className="text-muted-foreground" />}
        <a href={downloadUrl} target="_blank" rel="noopener" className="text-xs font-medium hover:underline">{file.name}</a>
        <span className="text-[10px] text-muted-foreground ml-auto">{formatBytes(file.size_bytes)}</span>
      </div>
      {isImage && (
        <img src={downloadUrl} alt={file.name} className="max-h-48 rounded border" />
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
