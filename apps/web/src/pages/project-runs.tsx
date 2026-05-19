import { useParams, Link } from "@tanstack/react-router";
import { useRuns } from "@/lib/queries/runs";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Play } from "lucide-react";

const STATUS_COLORS: Record<string, string> = {
  completed: "bg-green-50 text-green-700 border-green-200",
  running: "bg-blue-50 text-blue-700 border-blue-200",
  failed: "bg-red-50 text-red-700 border-red-200",
  pending: "bg-yellow-50 text-yellow-700 border-yellow-200",
};

export function ProjectRunsPage() {
  const { projectId } = useParams({ strict: false }) as { projectId: string };
  const { data: runs, isLoading } = useRuns(projectId);

  if (isLoading) return <p className="p-6 text-sm text-muted-foreground">Loading...</p>;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Runs</h1>
        <Button size="sm"><Play size={14} className="mr-1.5" />New Run</Button>
      </div>

      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Run ID</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Started</TableHead>
                <TableHead>Duration</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {(!runs || runs.length === 0) && (
                <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No runs yet</TableCell></TableRow>
              )}
              {runs?.map(run => {
                const duration = run.started_at && run.finished_at
                  ? `${Math.round((new Date(run.finished_at).getTime() - new Date(run.started_at).getTime()) / 1000)}s`
                  : run.status === "running" ? "Running..." : "—";
                return (
                  <TableRow key={run.id}>
                    <TableCell>
                      <Link
                        to="/projects/$projectId/runs/$runId"
                        params={{ projectId, runId: run.id }}
                        className="text-sm font-medium hover:underline"
                      >
                        {run.id.slice(0, 8)}
                      </Link>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className={STATUS_COLORS[run.status] ?? ""}>{run.status}</Badge>
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {run.started_at ? new Date(run.started_at).toLocaleString() : "—"}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">{duration}</TableCell>
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
