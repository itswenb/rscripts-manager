import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface ScriptMeta {
  title: string | null;
  description: string | null;
  inputs: { name: string; description: string | null }[];
  outputs: { name: string; description: string | null }[];
  params: { name: string; type: string; default: string | null; description: string | null }[];
}

export interface ScriptInfo {
  id: string;
  name: string;
  storage_path: string;
  meta: ScriptMeta;
}

export function useScripts() {
  return useQuery({
    queryKey: ["scripts"],
    queryFn: () => api.get<ScriptInfo[]>("/scripts"),
  });
}

export function useScript(assetId: string) {
  return useQuery({
    queryKey: ["scripts", assetId],
    queryFn: () => api.get<ScriptInfo>(`/scripts/${assetId}`),
    enabled: !!assetId,
  });
}
