import { useState } from "react";
import { useRuns, useCreateRun, useRunOutputs } from "@/lib/queries/runs";
import { useWorkflowSteps } from "@/lib/queries/workflow-steps";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

export function RunsPanel({ projectId }: { projectId: string }) {
  const { data: runs, isLoading } = useRuns(projectId);
  const { data: steps } = useWorkflowSteps();
  const createRun = useCreateRun(projectId);
  const [selectedStep, setSelectedStep] = useState("");
  const [expandedRun, setExpandedRun] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedStep) return;
    await createRun.mutateAsync({ workflow_step_id: selectedStep });
    setSelectedStep("");
  }

  if (isLoading) return <p>Loading...</p>;

  return (
    <div className="space-y-4">
      <form onSubmit={handleSubmit} className="flex gap-2 items-end">
        <div>
          <label className="text-sm font-medium">Workflow Step</label>
          <select
            value={selectedStep}
            onChange={(e) => setSelectedStep(e.target.value)}
            className="mt-1 block w-64 rounded-md border px-3 py-2 text-sm"
          >
            <option value="">Select a step...</option>
            {steps?.map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
        </div>
        <Button type="submit" disabled={!selectedStep || createRun.isPending}>
          Run
        </Button>
      </form>

      <div className="space-y-3">
        {runs?.map((run) => (
          <Card key={run.id}>
            <CardHeader className="py-3">
              <CardTitle className="text-sm flex items-center justify-between">
                <span>
                  <StatusBadge status={run.status} />
                  {" "}
                  {steps?.find((s) => s.id === run.workflow_step_id)?.name ?? run.workflow_step_id.slice(0, 8)}
                </span>
                <span className="text-xs text-muted-foreground">
                  {new Date(run.created_at).toLocaleString()}
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent className="py-2">
              <button
                className="text-xs text-blue-600 hover:underline"
                onClick={() => setExpandedRun(expandedRun === run.id ? null : run.id)}
              >
                {expandedRun === run.id ? "Hide details" : "Show details"}
              </button>
              {expandedRun === run.id && <RunDetails projectId={projectId} runId={run.id} run={run} />}
            </CardContent>
          </Card>
        ))}
        {runs?.length === 0 && <p className="text-muted-foreground text-sm">No runs yet</p>}
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    pending: "bg-yellow-100 text-yellow-800",
    running: "bg-blue-100 text-blue-800",
    completed: "bg-green-100 text-green-800",
    failed: "bg-red-100 text-red-800",
  };
  return (
    <span className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${colors[status] ?? ""}`}>
      {status}
    </span>
  );
}

function RunDetails({ projectId, runId, run }: { projectId: string; runId: string; run: any }) {
  const { data: outputs } = useRunOutputs(projectId, runId);

  return (
    <div className="mt-2 space-y-2 text-xs">
      {run.stdout && (
        <details>
          <summary className="cursor-pointer font-medium">stdout</summary>
          <pre className="mt-1 p-2 bg-muted rounded overflow-auto max-h-40">{run.stdout}</pre>
        </details>
      )}
      {run.stderr && (
        <details>
          <summary className="cursor-pointer font-medium">stderr</summary>
          <pre className="mt-1 p-2 bg-muted rounded overflow-auto max-h-40">{run.stderr}</pre>
        </details>
      )}
      {outputs && outputs.length > 0 && (
        <div>
          <p className="font-medium">Outputs:</p>
          <ul className="mt-1 space-y-1">
            {outputs.map((o) => (
              <li key={o.id}>
                <a
                  href={`/api/projects/${projectId}/runs/${runId}/outputs/${o.id}/download`}
                  className="text-blue-600 hover:underline"
                >
                  {o.name}
                </a>
                <span className="text-muted-foreground ml-2">({formatBytes(o.size_bytes)})</span>
              </li>
            ))}
          </ul>
        </div>
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
