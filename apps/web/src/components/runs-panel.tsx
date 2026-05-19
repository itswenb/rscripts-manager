import { useState } from "react";
import { useRuns, useCreateRun, useRunOutputs } from "@/lib/queries/runs";
import { useWorkflowSteps } from "@/lib/queries/workflow-steps";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Play, Eye } from "lucide-react";
import type { ScriptRun } from "@/lib/queries/runs";

const STATUS_VARIANTS: Record<string, "default" | "secondary" | "destructive" | "outline"> = {
  pending: "outline",
  running: "secondary",
  completed: "default",
  failed: "destructive",
};

const STATUS_CLASSES: Record<string, string> = {
  pending: "border-yellow-400 text-yellow-700 bg-yellow-50",
  running: "border-blue-400 text-blue-700 bg-blue-50",
  completed: "border-green-400 text-green-700 bg-green-50",
  failed: "border-red-400 text-red-700 bg-red-50",
};

export function RunsPanel({ projectId }: { projectId: string }) {
  const { data: runs, isLoading } = useRuns(projectId);
  const { data: steps } = useWorkflowSteps();
  const createRun = useCreateRun(projectId);
  const [selectedStep, setSelectedStep] = useState("");
  const [detailRun, setDetailRun] = useState<ScriptRun | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedStep) return;
    await createRun.mutateAsync({ workflow_step_id: selectedStep });
    setSelectedStep("");
  }

  function duration(run: ScriptRun): string {
    if (!run.started_at) return "—";
    const end = run.finished_at ? new Date(run.finished_at) : new Date();
    const ms = end.getTime() - new Date(run.started_at).getTime();
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  if (isLoading) return <p className="text-sm text-muted-foreground">Loading...</p>;

  return (
    <div className="space-y-4">
      <form onSubmit={handleSubmit} className="flex items-end gap-3">
        <div className="space-y-1.5 w-64">
          <Label>Workflow Step</Label>
          <Select value={selectedStep} onValueChange={setSelectedStep}>
            <SelectTrigger>
              <SelectValue placeholder="Select a step..." />
            </SelectTrigger>
            <SelectContent>
              {steps?.map((s) => (
                <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <Button type="submit" size="sm" disabled={!selectedStep || createRun.isPending}>
          <Play size={13} className="mr-1.5" />
          Run
        </Button>
      </form>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-28">Status</TableHead>
              <TableHead>Workflow Step</TableHead>
              <TableHead>Started</TableHead>
              <TableHead className="w-24">Duration</TableHead>
              <TableHead className="w-10"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {runs?.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">No runs yet</TableCell>
              </TableRow>
            )}
            {runs?.map((run) => (
              <TableRow key={run.id}>
                <TableCell>
                  <Badge className={STATUS_CLASSES[run.status] ?? ""} variant="outline">
                    {run.status}
                  </Badge>
                </TableCell>
                <TableCell className="text-sm">
                  {steps?.find((s) => s.id === run.workflow_step_id)?.name ?? run.workflow_step_id.slice(0, 8)}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">
                  {run.started_at ? new Date(run.started_at).toLocaleString() : "—"}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{duration(run)}</TableCell>
                <TableCell>
                  <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setDetailRun(run)}>
                    <Eye size={13} />
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <Dialog open={!!detailRun} onOpenChange={(o) => !o && setDetailRun(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Run Details</DialogTitle>
          </DialogHeader>
          {detailRun && <RunDetails projectId={projectId} run={detailRun} />}
        </DialogContent>
      </Dialog>
    </div>
  );
}

function RunDetails({ projectId, run }: { projectId: string; run: ScriptRun }) {
  const { data: outputs } = useRunOutputs(projectId, run.id);
  return (
    <div className="space-y-3 text-sm">
      {run.stdout && (
        <div>
          <p className="font-medium mb-1 text-xs text-muted-foreground uppercase tracking-wide">stdout</p>
          <pre className="p-3 bg-muted rounded-md overflow-auto max-h-48 text-xs">{run.stdout}</pre>
        </div>
      )}
      {run.stderr && (
        <div>
          <p className="font-medium mb-1 text-xs text-muted-foreground uppercase tracking-wide">stderr</p>
          <pre className="p-3 bg-muted rounded-md overflow-auto max-h-48 text-xs text-destructive">{run.stderr}</pre>
        </div>
      )}
      {outputs && outputs.length > 0 && (
        <div>
          <p className="font-medium mb-1 text-xs text-muted-foreground uppercase tracking-wide">Outputs</p>
          <ul className="space-y-1">
            {outputs.map((o) => (
              <li key={o.id} className="flex items-center gap-2">
                <a href={`/api/projects/${projectId}/runs/${run.id}/outputs/${o.id}/download`} className="text-primary hover:underline">
                  {o.name}
                </a>
                <span className="text-muted-foreground text-xs">({formatBytes(o.size_bytes)})</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {!run.stdout && !run.stderr && (!outputs || outputs.length === 0) && (
        <p className="text-muted-foreground">No output available</p>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
