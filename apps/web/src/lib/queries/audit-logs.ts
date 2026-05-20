import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface AuditLog {
  id: string;
  user_id: string | null;
  username: string;
  action: string;
  resource_type: string;
  resource_id: string | null;
  details: unknown;
  created_at: string;
}

export interface AuditLogFilters {
  username?: string;
  action?: string;
  resource_type?: string;
  limit?: number;
  offset?: number;
}

export function useAuditLogs(filters: AuditLogFilters = {}) {
  const params = new URLSearchParams();
  if (filters.username) params.set("username", filters.username);
  if (filters.action) params.set("action", filters.action);
  if (filters.resource_type) params.set("resource_type", filters.resource_type);
  if (filters.limit) params.set("limit", String(filters.limit));
  if (filters.offset) params.set("offset", String(filters.offset));
  const qs = params.toString();
  return useQuery({
    queryKey: ["audit-logs", filters],
    queryFn: () => api.get<AuditLog[]>(`/audit-logs${qs ? `?${qs}` : ""}`),
  });
}
