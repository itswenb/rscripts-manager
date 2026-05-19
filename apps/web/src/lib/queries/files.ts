import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface FileAsset {
  id: string;
  project_id: string;
  parent_id: string | null;
  name: string;
  is_directory: boolean;
  size_bytes: number;
  mime_type: string | null;
  storage_path: string;
  created_at: string;
}

export function useFiles(projectId: string, parentId?: string | null) {
  const params = parentId ? `?parent_id=${parentId}` : "";
  return useQuery({
    queryKey: ["files", projectId, parentId ?? "root"],
    queryFn: () => api.get<FileAsset[]>(`/projects/${projectId}/files${params}`),
  });
}

export function useUploadFiles(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { files: FileList; parentId?: string }) => {
      const form = new FormData();
      for (const file of Array.from(data.files)) {
        form.append("files", file);
      }
      const params = data.parentId ? `?parent_id=${data.parentId}` : "";
      return api.post<FileAsset[]>(`/projects/${projectId}/files${params}`, form);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}

export function useCreateDirectory(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; parent_id?: string }) =>
      api.post<FileAsset>(`/projects/${projectId}/files/directory`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}

export function useDeleteFile(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (assetId: string) => api.delete(`/projects/${projectId}/files/${assetId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}
