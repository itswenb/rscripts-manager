import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useWorkflowSteps, useCreateWorkflowStep, useDeleteWorkflowStep } from "@/lib/queries/workflow-steps";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Plus, MoreHorizontal, Trash2 } from "lucide-react";

export function WorkflowStepsPage() {
  const { t } = useTranslation();
  const { data: steps, isLoading } = useWorkflowSteps();
  const createStep = useCreateWorkflowStep();
  const deleteStep = useDeleteWorkflowStep();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [scriptPath, setScriptPath] = useState("");
  const [description, setDescription] = useState("");
  const [outputDir, setOutputDir] = useState("outputs");
  const [params, setParams] = useState("");

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || !scriptPath.trim()) return;
    let paramSchema: unknown[] = [];
    if (params.trim()) {
      try { paramSchema = JSON.parse(params.trim()); } catch { /* ignore */ }
    }
    await createStep.mutateAsync({
      name: name.trim(),
      script_path: scriptPath.trim(),
      description: description.trim() || undefined,
      output_dir_name: outputDir.trim() || "outputs",
      param_schema: paramSchema,
    });
    setName(""); setScriptPath(""); setDescription(""); setOutputDir("outputs"); setParams("");
    setOpen(false);
  }

  if (isLoading) return <p className="p-6 text-sm text-muted-foreground">{t("common.loading")}</p>;

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h1 className="text-lg font-semibold">{t("workflows.title")}</h1>
          {steps && steps.length > 0 && (
            <Badge variant="secondary">{steps.length}</Badge>
          )}
        </div>
        <Button size="sm" onClick={() => setOpen(true)}>
          <Plus size={14} className="mr-1.5" />
          {t("workflows.addStep")}
        </Button>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t("workflows.addStep")}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreate} className="space-y-4 pt-2">
            <div className="space-y-1.5">
              <Label>{t("workflows.stepName")}</Label>
              <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus />
            </div>
            <div className="space-y-1.5">
              <Label>{t("workflows.script")}</Label>
              <Input value={scriptPath} onChange={(e) => setScriptPath(e.target.value)} placeholder="scripts/analysis.R" />
              <p className="text-xs text-muted-foreground">{t("workflows.scriptHint")}</p>
            </div>
            <div className="space-y-1.5">
              <Label>{t("workflows.outputDir")}</Label>
              <Input value={outputDir} onChange={(e) => setOutputDir(e.target.value)} placeholder="outputs" />
            </div>
            <div className="space-y-1.5">
              <Label>{t("workflows.params")}</Label>
              <Textarea value={params} onChange={(e) => setParams(e.target.value)} rows={3} placeholder='[{"name":"alpha","type":"number","default":0.05}]' />
              <p className="text-xs text-muted-foreground">{t("workflows.paramsHint")}</p>
            </div>
            <div className="space-y-1.5">
              <Label>{t("common.description")}</Label>
              <Textarea value={description} onChange={(e) => setDescription(e.target.value)} rows={2} />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setOpen(false)}>{t("common.cancel")}</Button>
              <Button type="submit" disabled={createStep.isPending || !name.trim() || !scriptPath.trim()}>{t("common.create")}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("common.name")}</TableHead>
              <TableHead>{t("workflows.script")}</TableHead>
              <TableHead>{t("workflows.outputDir")}</TableHead>
              <TableHead>{t("common.description")}</TableHead>
              <TableHead className="w-10"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {steps?.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">{t("workflows.noSteps")}</TableCell>
              </TableRow>
            )}
            {steps?.map((s) => (
              <TableRow key={s.id}>
                <TableCell className="font-medium">{s.name}</TableCell>
                <TableCell className="font-mono text-xs text-muted-foreground">{s.script_path}</TableCell>
                <TableCell className="text-xs text-muted-foreground">{s.output_dir_name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{s.description || "—"}</TableCell>
                <TableCell>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <button className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground">
                        <MoreHorizontal size={14} />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        className="text-destructive focus:text-destructive"
                        onClick={() => deleteStep.mutate(s.id)}
                      >
                        <Trash2 size={13} className="mr-2" />
                        {t("common.delete")}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
