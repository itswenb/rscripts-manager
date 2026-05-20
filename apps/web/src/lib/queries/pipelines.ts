import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface Pipeline {
  id: string;
  project_id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface PipelineStep {
  id: string;
  pipeline_id: string;
  step_order: number;
  script_path: string;
  label: string;
  param_values: Record<string, unknown>;
  created_at: string;
}

export interface PipelineWithSteps extends Pipeline {
  steps: PipelineStep[];
}

export interface PipelineRun {
  id: string;
  pipeline_id: string;
  project_id: string;
  status: "pending" | "running" | "completed" | "failed";
  current_step: number;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
}

export interface PipelineStepRun {
  id: string;
  pipeline_run_id: string;
  step_order: number;
  script_path: string;
  status: "pending" | "running" | "completed" | "failed";
  stdout: string | null;
  stderr: string | null;
  started_at: string | null;
  finished_at: string | null;
}

export interface PipelineRunDetail extends PipelineRun {
  step_runs: PipelineStepRun[];
}

export interface StepOutputFile {
  id: string;
  step_run_id: string;
  name: string;
  size_bytes: number;
  mime_type: string | null;
  storage_path: string;
  created_at: string;
}

export function usePipelines(projectId: string) {
  return useQuery({
    queryKey: ["pipelines", projectId],
    queryFn: () => api.get<Pipeline[]>(`/projects/${projectId}/pipelines`),
    enabled: !!projectId,
  });
}

export function usePipeline(projectId: string, pipelineId: string) {
  return useQuery({
    queryKey: ["pipelines", projectId, pipelineId],
    queryFn: () => api.get<PipelineWithSteps>(`/projects/${projectId}/pipelines/${pipelineId}`),
    enabled: !!projectId && !!pipelineId,
  });
}

export function useCreatePipeline(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: {
      name: string;
      description?: string;
      steps: { script_path: string; label?: string; param_values?: Record<string, unknown> }[];
    }) => api.post<PipelineWithSteps>(`/projects/${projectId}/pipelines`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["pipelines", projectId] }),
  });
}

export function useDeletePipeline(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (pipelineId: string) =>
      api.delete(`/projects/${projectId}/pipelines/${pipelineId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["pipelines", projectId] }),
  });
}

export function useStartPipelineRun(projectId: string, pipelineId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data?: { input_files?: string[]; param_overrides?: Record<string, unknown> }) =>
      api.post<PipelineRun>(`/projects/${projectId}/pipelines/${pipelineId}/runs`, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pipeline-runs", projectId, pipelineId] });
    },
  });
}

export function usePipelineRuns(projectId: string, pipelineId: string) {
  return useQuery({
    queryKey: ["pipeline-runs", projectId, pipelineId],
    queryFn: () => api.get<PipelineRun[]>(`/projects/${projectId}/pipelines/${pipelineId}/runs`),
    enabled: !!projectId && !!pipelineId,
    refetchInterval: (query) => {
      const data = query.state.data;
      const hasActive = data?.some((r) => r.status === "pending" || r.status === "running");
      return hasActive ? 3000 : false;
    },
  });
}

export function usePipelineRunDetail(projectId: string, pipelineId: string, runId: string) {
  return useQuery({
    queryKey: ["pipeline-runs", projectId, pipelineId, runId],
    queryFn: () =>
      api.get<PipelineRunDetail>(`/projects/${projectId}/pipelines/${pipelineId}/runs/${runId}`),
    enabled: !!runId,
    refetchInterval: (query) => {
      const data = query.state.data;
      return data?.status === "running" || data?.status === "pending" ? 2000 : false;
    },
  });
}

export function useStepOutputs(projectId: string, pipelineId: string, runId: string, stepRunId: string) {
  return useQuery({
    queryKey: ["step-outputs", stepRunId],
    queryFn: () =>
      api.get<StepOutputFile[]>(
        `/projects/${projectId}/pipelines/${pipelineId}/runs/${runId}/steps/${stepRunId}/outputs`
      ),
    enabled: !!stepRunId,
  });
}
