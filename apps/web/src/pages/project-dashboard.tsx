import { useParams } from "@tanstack/react-router";
import { useProject } from "@/lib/queries/projects";
import { useRuns } from "@/lib/queries/runs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { Progress } from "@/components/ui/progress";
import { Play, Upload, Activity, HardDrive, AlertTriangle, CheckCircle } from "lucide-react";

const STATUS_COLORS: Record<string, string> = {
  completed: "bg-green-50 text-green-700 border-green-200",
  running: "bg-blue-50 text-blue-700 border-blue-200",
  failed: "bg-red-50 text-red-700 border-red-200",
  pending: "bg-yellow-50 text-yellow-700 border-yellow-200",
};

export function ProjectDashboardPage() {
  const { projectId } = useParams({ strict: false }) as { projectId: string };
  const { data: project } = useProject(projectId);
  const { data: runs } = useRuns(projectId);

  const now = Date.now();
  const day = 86400000;
  const failedLast24h = runs?.filter(r => r.status === "failed" && r.created_at && now - new Date(r.created_at).getTime() < day).length ?? 0;
  const activeWorkflows = new Set(runs?.filter(r => r.status === "running").map(r => r.workflow_step_id)).size;

  const chartData = Array.from({ length: 7 }, (_, i) => {
    const date = new Date(now - (6 - i) * day);
    const label = date.toLocaleDateString(undefined, { weekday: "short" });
    const count = runs?.filter(r => {
      const d = new Date(r.created_at);
      return d.toDateString() === date.toDateString();
    }).length ?? 0;
    return { name: label, runs: count };
  });

  const recentRuns = runs?.slice(0, 5) ?? [];

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">{project?.name ?? "Project"}</h1>
          <Badge variant="outline">v1.0</Badge>
        </div>
        <Button size="sm"><Play size={14} className="mr-1.5" />Add New Run</Button>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1"><CheckCircle size={13} />System Status</div>
            <p className="text-lg font-semibold text-green-600">Operational</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1"><Activity size={13} />Active Workflows</div>
            <p className="text-lg font-semibold">{activeWorkflows}</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1"><AlertTriangle size={13} />Failed (24h)</div>
            <p className="text-lg font-semibold text-red-600">{failedLast24h}</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1"><HardDrive size={13} />Total Runs</div>
            <p className="text-lg font-semibold">{runs?.length ?? 0}</p>
          </CardContent>
        </Card>
      </div>

      <div className="flex gap-2">
        <Button variant="outline" size="sm"><Play size={13} className="mr-1.5" />Start New Run</Button>
        <Button variant="outline" size="sm"><Upload size={13} className="mr-1.5" />Upload Datasets</Button>
      </div>

      <div className="grid lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2">
          <CardHeader className="pb-2"><CardTitle className="text-sm">Runs (Last 7 Days)</CardTitle></CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="name" className="text-xs" />
                <YAxis allowDecimals={false} className="text-xs" />
                <Tooltip />
                <Line type="monotone" dataKey="runs" stroke="hsl(var(--primary))" strokeWidth={2} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm">Storage</CardTitle></CardHeader>
          <CardContent className="space-y-3">
            <Progress value={35} />
            <p className="text-xs text-muted-foreground">3.5 GB of 10 GB used</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader className="pb-2"><CardTitle className="text-sm">Recent Executions</CardTitle></CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Run</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Started</TableHead>
                <TableHead>Duration</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentRuns.length === 0 && (
                <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-6">No runs yet</TableCell></TableRow>
              )}
              {recentRuns.map(run => {
                const duration = run.started_at && run.finished_at
                  ? `${Math.round((new Date(run.finished_at).getTime() - new Date(run.started_at).getTime()) / 1000)}s`
                  : run.status === "running" ? "Running..." : "—";
                return (
                  <TableRow key={run.id}>
                    <TableCell className="font-mono text-xs">{run.id.slice(0, 8)}</TableCell>
                    <TableCell>
                      <Badge variant="outline" className={STATUS_COLORS[run.status] ?? ""}>
                        {run.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {run.started_at ? new Date(run.started_at).toLocaleString() : "—"}
                    </TableCell>
                    <TableCell className="text-xs">{duration}</TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
