import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface ScriptRun {
  id: string;
  project_id: string;
  workflow_step_id: string;
  status: "pending" | "running" | "completed" | "failed";
  inputs: Record<string, unknown>;
  params: Record<string, unknown>;
  stdout: string | null;
  stderr: string | null;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
}

export interface OutputFile {
  id: string;
  run_id: string;
  name: string;
  size_bytes: number;
  mime_type: string | null;
  storage_path: string;
  created_at: string;
}

export function useRuns(projectId: string) {
  return useQuery({
    queryKey: ["runs", projectId],
    queryFn: () => api.get<ScriptRun[]>(`/projects/${projectId}/runs`),
    refetchInterval: (query) => {
      const data = query.state.data;
      const hasActive = data?.some((r) => r.status === "pending" || r.status === "running");
      return hasActive ? 3000 : false;
    },
  });
}

export function useRun(projectId: string, runId: string) {
  return useQuery({
    queryKey: ["runs", projectId, runId],
    queryFn: () => api.get<ScriptRun>(`/projects/${projectId}/runs/${runId}`),
    refetchInterval: (query) => {
      const data = query.state.data;
      return data?.status === "running" || data?.status === "pending" ? 2000 : false;
    },
  });
}

export function useCreateRun(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { workflow_step_id: string; inputs?: object; params?: object }) =>
      api.post<ScriptRun>(`/projects/${projectId}/runs`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["runs", projectId] }),
  });
}

export function useRunOutputs(projectId: string, runId: string) {
  return useQuery({
    queryKey: ["runs", projectId, runId, "outputs"],
    queryFn: () => api.get<OutputFile[]>(`/projects/${projectId}/runs/${runId}/outputs`),
    enabled: !!runId,
  });
}
