import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface UserFile {
  id: string;
  project_id: string | null;
  parent_id: string | null;
  owner_id: string | null;
  is_public: boolean;
  name: string;
  is_directory: boolean;
  size_bytes: number;
  mime_type: string | null;
  storage_path: string;
  created_at: string;
}

export function useMyFiles(parentId?: string | null) {
  const params = parentId ? `?parent_id=${parentId}` : "";
  return useQuery({
    queryKey: ["my-files", parentId ?? "root"],
    queryFn: () => api.get<UserFile[]>(`/my-files${params}`),
  });
}

export function usePublicFiles(parentId?: string | null) {
  const params = parentId ? `?parent_id=${parentId}` : "";
  return useQuery({
    queryKey: ["public-files", parentId ?? "root"],
    queryFn: () => api.get<UserFile[]>(`/public-files${params}`),
  });
}

export function useUploadMyFiles() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { files: FileList; parentId?: string }) => {
      const form = new FormData();
      for (const file of Array.from(data.files)) {
        form.append("files", file);
      }
      const params = data.parentId ? `?parent_id=${data.parentId}` : "";
      return api.post<UserFile[]>(`/my-files${params}`, form);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-files"] }),
  });
}

export function useCreateMyDirectory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; parent_id?: string }) =>
      api.post<UserFile>("/my-files/directory", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-files"] }),
  });
}

export function useDeleteFile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (assetId: string) => api.delete(`/files/${assetId}`),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["my-files"] });
      qc.invalidateQueries({ queryKey: ["public-files"] });
    },
  });
}

export function useRenameFile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      api.post<UserFile>(`/files/${id}/rename`, { name }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["my-files"] });
      qc.invalidateQueries({ queryKey: ["public-files"] });
    },
  });
}

export function useMoveFile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, parentId }: { id: string; parentId: string | null }) =>
      api.post<UserFile>(`/files/${id}/move`, { parent_id: parentId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-files"] }),
  });
}

export function useCopyFile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, parentId }: { id: string; parentId: string | null }) =>
      api.post<UserFile>(`/files/${id}/copy`, { parent_id: parentId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-files"] }),
  });
}

export function useMoveToPublic() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (assetId: string) => api.post<UserFile>(`/files/${assetId}/move-to-public`),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["my-files"] });
      qc.invalidateQueries({ queryKey: ["public-files"] });
    },
  });
}
