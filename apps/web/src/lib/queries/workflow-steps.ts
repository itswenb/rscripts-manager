import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface WorkflowStep {
  id: string;
  name: string;
  description: string;
  script_path: string;
  input_schema: unknown[];
  param_schema: unknown[];
  output_dir_name: string;
  created_at: string;
}

export function useWorkflowSteps() {
  return useQuery({
    queryKey: ["workflow-steps"],
    queryFn: () => api.get<WorkflowStep[]>("/workflow-steps"),
  });
}

export function useCreateWorkflowStep() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; script_path: string; description?: string; output_dir_name?: string; param_schema?: unknown[] }) =>
      api.post<WorkflowStep>("/workflow-steps", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workflow-steps"] }),
  });
}

export function useDeleteWorkflowStep() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/workflow-steps/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workflow-steps"] }),
  });
}
