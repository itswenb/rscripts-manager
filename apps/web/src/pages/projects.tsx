import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { useProjects, useCreateProject, useDeleteProject } from "@/lib/queries/projects";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Plus, MoreHorizontal, Trash2, FileText } from "lucide-react";

const STATUS_CONFIG = {
  success: { label: "Success", class: "bg-green-50 text-green-700 border-green-200" },
  running: { label: "Running", class: "bg-blue-50 text-blue-700 border-blue-200" },
  failed: { label: "Failed", class: "bg-red-50 text-red-700 border-red-200" },
  idle: { label: "Idle", class: "bg-gray-50 text-gray-500 border-gray-200" },
} as const;

export function ProjectsPage() {
  const { t } = useTranslation();
  const { data: projects, isLoading } = useProjects();
  const createProject = useCreateProject();
  const deleteProject = useDeleteProject();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    await createProject.mutateAsync({ name: name.trim(), description: description.trim() || undefined });
    setName("");
    setDescription("");
    setOpen(false);
  }

  if (isLoading) return <p className="p-6 text-sm text-muted-foreground">{t("common.loading")}</p>;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">{t("projects.title")}</h1>
        <Button size="sm" onClick={() => setOpen(true)}>
          <Plus size={14} className="mr-1.5" />
          {t("projects.addNew")}
        </Button>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t("projects.addNew")}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreate} className="space-y-4 pt-2">
            <div className="space-y-1.5">
              <Label htmlFor="proj-name">{t("common.name")}</Label>
              <Input id="proj-name" value={name} onChange={(e) => setName(e.target.value)} autoFocus />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="proj-desc">{t("common.description")}</Label>
              <Textarea id="proj-desc" value={description} onChange={(e) => setDescription(e.target.value)} rows={2} />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setOpen(false)}>{t("common.cancel")}</Button>
              <Button type="submit" disabled={createProject.isPending || !name.trim()}>{t("common.create")}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {projects?.length === 0 ? (
        <div className="text-center py-12 text-muted-foreground text-sm">{t("projects.noProjects")}</div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {projects?.map((p, i) => (
            <Card key={p.id} className="relative group hover:shadow-md transition-shadow">
              <CardContent className="p-5 space-y-3">
                <div className="flex items-start justify-between">
                  <Badge variant="outline" className={STATUS_CONFIG.idle.class}>
                    {STATUS_CONFIG.idle.label}
                  </Badge>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="icon" className="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity">
                        <MoreHorizontal size={14} />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        className="text-destructive focus:text-destructive"
                        onClick={() => deleteProject.mutate(p.id)}
                      >
                        <Trash2 size={13} className="mr-2" />
                        {t("common.delete")}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">PRJ-{String(i + 1).padStart(3, "0")}</p>
                  <Link to="/projects/$projectId" params={{ projectId: p.id }} className="text-sm font-medium hover:underline">
                    {p.name}
                  </Link>
                </div>
                <p className="text-xs text-muted-foreground line-clamp-2">{p.description || "No description"}</p>
                <div className="flex items-center justify-between text-xs text-muted-foreground pt-1 border-t">
                  <span className="flex items-center gap-1"><FileText size={12} /> Files</span>
                  <span>{new Date(p.updated_at).toLocaleDateString()}</span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
