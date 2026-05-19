import { useState } from "react";
import { useParams } from "@tanstack/react-router";
import { useRun, useRunOutputs } from "@/lib/queries/runs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Download, Maximize2, ArrowDown } from "lucide-react";

const STATUS_COLORS: Record<string, string> = {
  completed: "bg-green-50 text-green-700 border-green-200",
  running: "bg-blue-50 text-blue-700 border-blue-200",
  failed: "bg-red-50 text-red-700 border-red-200",
  pending: "bg-yellow-50 text-yellow-700 border-yellow-200",
};

const LOG_COLORS: Record<string, string> = {
  INFO: "text-blue-600",
  OK: "text-green-600",
  WARN: "text-yellow-600",
  ERROR: "text-red-600",
};

function parseLogLine(line: string) {
  const match = line.match(/^\[(\d{2}:\d{2}:\d{2})\]\s*\[(\w+)\]\s*(.*)/);
  if (!match) return { time: "", level: "INFO", message: line };
  return { time: match[1], level: match[2], message: match[3] };
}

export function RunDetailPage() {
  const { projectId, runId } = useParams({ strict: false }) as { projectId: string; runId: string };
  const { data: run, isLoading } = useRun(projectId, runId);
  const { data: outputs } = useRunOutputs(projectId, runId);
  const [autoScroll, setAutoScroll] = useState(true);

  if (isLoading) return <p className="p-6 text-sm text-muted-foreground">Loading...</p>;
  if (!run) return <p className="p-6 text-sm text-muted-foreground">Run not found</p>;

  const duration = run.started_at && run.finished_at
    ? `${Math.round((new Date(run.finished_at).getTime() - new Date(run.started_at).getTime()) / 1000)}s`
    : run.status === "running" ? "Running..." : "—";

  const logLines = run.stdout?.split("\n").filter(Boolean) ?? [];

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs text-muted-foreground mb-1">Run {run.id.slice(0, 8)}</p>
          <h1 className="text-lg font-semibold">Execution Detail</h1>
        </div>
        {run.status === "running" && <Button variant="destructive" size="sm">Abort</Button>}
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card><CardContent className="p-4">
          <p className="text-xs text-muted-foreground mb-1">Status</p>
          <Badge variant="outline" className={STATUS_COLORS[run.status] ?? ""}>{run.status}</Badge>
        </CardContent></Card>
        <Card><CardContent className="p-4">
          <p className="text-xs text-muted-foreground mb-1">Started</p>
          <p className="text-sm font-medium">{run.started_at ? new Date(run.started_at).toLocaleString() : "—"}</p>
        </CardContent></Card>
        <Card><CardContent className="p-4">
          <p className="text-xs text-muted-foreground mb-1">Duration</p>
          <p className="text-sm font-medium">{duration}</p>
        </CardContent></Card>
        <Card><CardContent className="p-4">
          <p className="text-xs text-muted-foreground mb-1">Compute Node</p>
          <p className="text-sm font-medium">default</p>
        </CardContent></Card>
      </div>

      {run.status === "running" && (
        <div className="space-y-2">
          <p className="text-sm text-muted-foreground">Processing...</p>
          <Progress value={undefined} className="animate-pulse" />
        </div>
      )}

      <Tabs defaultValue="logs">
        <TabsList>
          <TabsTrigger value="logs">Live Logs</TabsTrigger>
          <TabsTrigger value="results">Results</TabsTrigger>
          <TabsTrigger value="config">Configuration</TabsTrigger>
        </TabsList>
        <TabsContent value="logs" className="mt-4">
          <Card>
            <CardHeader className="pb-2 flex-row items-center justify-between">
              <CardTitle className="text-sm">Output</CardTitle>
              <div className="flex gap-2">
                <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setAutoScroll(!autoScroll)} title="Auto-scroll">
                  <ArrowDown size={13} className={autoScroll ? "text-primary" : ""} />
                </Button>
                <Button variant="ghost" size="icon" className="h-7 w-7" title="Download logs">
                  <Download size={13} />
                </Button>
                <Button variant="ghost" size="icon" className="h-7 w-7" title="Fullscreen">
                  <Maximize2 size={13} />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="bg-muted/50 rounded-md p-3 max-h-80 overflow-auto font-mono text-xs space-y-0.5">
                {logLines.length === 0 && <p className="text-muted-foreground">No output yet</p>}
                {logLines.map((line, i) => {
                  const { time, level, message } = parseLogLine(line);
                  return (
                    <div key={i} className="flex gap-2">
                      {time && <span className="text-muted-foreground shrink-0">{time}</span>}
                      {time && <span className={`shrink-0 ${LOG_COLORS[level] ?? ""}`}>[{level}]</span>}
                      <span>{message}</span>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="results" className="mt-4">
          <Card>
            <CardContent className="p-4">
              {!outputs?.length ? (
                <p className="text-sm text-muted-foreground">No output files</p>
              ) : (
                <ul className="space-y-2">
                  {outputs.map(f => (
                    <li key={f.id} className="flex items-center justify-between text-sm">
                      <span>{f.name}</span>
                      <a href={`/api/projects/${projectId}/runs/${runId}/outputs/${f.id}/download`} className="text-primary hover:underline text-xs">Download</a>
                    </li>
                  ))}
                </ul>
              )}
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="config" className="mt-4">
          <Card>
            <CardContent className="p-4">
              <pre className="text-xs bg-muted/50 rounded-md p-3 overflow-auto max-h-60">
                {JSON.stringify({ inputs: run.inputs, params: run.params }, null, 2)}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {run.stderr && (
        <Card className="border-red-200">
          <CardHeader className="pb-2"><CardTitle className="text-sm text-red-600">Errors</CardTitle></CardHeader>
          <CardContent>
            <pre className="text-xs font-mono text-red-600 bg-red-50 rounded-md p-3 overflow-auto max-h-40">{run.stderr}</pre>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
