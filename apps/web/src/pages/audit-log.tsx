import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuditLogs, AuditLogFilters } from "@/lib/queries/audit-logs";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";

export function AuditLogPage() {
  const { t } = useTranslation();
  const [filters, setFilters] = useState<AuditLogFilters>({ limit: 100 });
  const { data: logs, isLoading } = useAuditLogs(filters);

  return (
    <div className="p-6 space-y-4">
      <h1 className="text-lg font-semibold">{t("audit.title")}</h1>

      <div className="flex items-center gap-3">
        <Input
          placeholder={t("audit.user")}
          className="w-40"
          value={filters.username ?? ""}
          onChange={(e) => setFilters({ ...filters, username: e.target.value || undefined })}
        />
        <select
          className="border rounded-md px-3 py-1.5 text-sm bg-background"
          value={filters.action ?? ""}
          onChange={(e) => setFilters({ ...filters, action: e.target.value || undefined })}
        >
          <option value="">{t("audit.action")}</option>
          <option value="create">{t("audit.actions.create")}</option>
          <option value="update">{t("audit.actions.update")}</option>
          <option value="delete">{t("audit.actions.delete")}</option>
          <option value="upload">{t("audit.actions.upload")}</option>
          <option value="download">{t("audit.actions.download")}</option>
          <option value="rename">{t("audit.actions.rename")}</option>
          <option value="move">{t("audit.actions.move")}</option>
          <option value="move_to_public">{t("audit.actions.move_to_public")}</option>
        </select>
        <select
          className="border rounded-md px-3 py-1.5 text-sm bg-background"
          value={filters.resource_type ?? ""}
          onChange={(e) => setFilters({ ...filters, resource_type: e.target.value || undefined })}
        >
          <option value="">{t("audit.resource")}</option>
          <option value="project">Project</option>
          <option value="file">File</option>
          <option value="directory">Directory</option>
          <option value="user">User</option>
          <option value="workflow_step">Workflow Step</option>
          <option value="run">Run</option>
        </select>
      </div>

      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("audit.user")}</TableHead>
              <TableHead>{t("audit.action")}</TableHead>
              <TableHead>{t("audit.resource")}</TableHead>
              <TableHead>{t("audit.timestamp")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {logs?.map((log) => (
              <TableRow key={log.id}>
                <TableCell className="font-medium">{log.username}</TableCell>
                <TableCell>
                  <Badge variant="outline">{log.action}</Badge>
                </TableCell>
                <TableCell className="text-muted-foreground text-xs">
                  {log.resource_type}{log.resource_id ? ` / ${log.resource_id.slice(0, 8)}` : ""}
                </TableCell>
                <TableCell className="text-muted-foreground text-xs">
                  {new Date(log.created_at).toLocaleString()}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  );
}
