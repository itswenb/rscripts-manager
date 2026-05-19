import { useState } from "react";
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
  const { data: steps, isLoading } = useWorkflowSteps();
  const createStep = useCreateWorkflowStep();
  const deleteStep = useDeleteWorkflowStep();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [scriptPath, setScriptPath] = useState("");
  const [description, setDescription] = useState("");

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || !scriptPath.trim()) return;
    await createStep.mutateAsync({ name: name.trim(), script_path: scriptPath.trim(), description: description.trim() || undefined });
    setName("");
    setScriptPath("");
    setDescription("");
    setOpen(false);
  }

  if (isLoading) return <p className="text-sm text-muted-foreground">Loading...</p>;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h1 className="text-lg font-semibold">Workflow Steps</h1>
          {steps && steps.length > 0 && (
            <Badge variant="secondary">{steps.length}</Badge>
          )}
        </div>
        <Button size="sm" onClick={() => setOpen(true)}>
          <Plus size={14} className="mr-1.5" />
          Register Step
        </Button>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Register Workflow Step</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreate} className="space-y-4 pt-2">
            <div className="space-y-1.5">
              <Label htmlFor="step-name">Name</Label>
              <Input id="step-name" value={name} onChange={(e) => setName(e.target.value)} autoFocus />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="step-path">Script Path</Label>
              <Input id="step-path" value={scriptPath} onChange={(e) => setScriptPath(e.target.value)} placeholder="/scripts/analysis.R" />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="step-desc">Description</Label>
              <Textarea id="step-desc" value={description} onChange={(e) => setDescription(e.target.value)} rows={2} />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
              <Button type="submit" disabled={createStep.isPending || !name.trim() || !scriptPath.trim()}>Register</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Script Path</TableHead>
              <TableHead>Description</TableHead>
              <TableHead className="w-10"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {steps?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground py-8">No workflow steps registered</TableCell>
              </TableRow>
            )}
            {steps?.map((s) => (
              <TableRow key={s.id}>
                <TableCell className="font-medium">{s.name}</TableCell>
                <TableCell className="font-mono text-xs text-muted-foreground">{s.script_path}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{s.description || "—"}</TableCell>
                <TableCell>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="icon" className="h-7 w-7">
                        <MoreHorizontal size={14} />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        className="text-destructive focus:text-destructive"
                        onClick={() => deleteStep.mutate(s.id)}
                      >
                        <Trash2 size={13} className="mr-2" />
                        Delete
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
