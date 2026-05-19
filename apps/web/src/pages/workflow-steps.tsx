import { useState } from "react";
import { useWorkflowSteps, useCreateWorkflowStep, useDeleteWorkflowStep } from "@/lib/queries/workflow-steps";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

export function WorkflowStepsPage() {
  const { data: steps, isLoading } = useWorkflowSteps();
  const createStep = useCreateWorkflowStep();
  const deleteStep = useDeleteWorkflowStep();
  const [name, setName] = useState("");
  const [scriptPath, setScriptPath] = useState("");

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || !scriptPath.trim()) return;
    await createStep.mutateAsync({ name: name.trim(), script_path: scriptPath.trim() });
    setName("");
    setScriptPath("");
  }

  if (isLoading) return <p>Loading...</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Workflow Steps</h1>

      <form onSubmit={handleCreate} className="flex gap-2">
        <Input placeholder="Step name" value={name} onChange={(e) => setName(e.target.value)} />
        <Input placeholder="Script path" value={scriptPath} onChange={(e) => setScriptPath(e.target.value)} />
        <Button type="submit" disabled={createStep.isPending}>Register</Button>
      </form>

      <div className="space-y-3">
        {steps?.map((s) => (
          <Card key={s.id}>
            <CardHeader className="py-3">
              <CardTitle className="text-sm flex items-center justify-between">
                <span>{s.name}</span>
                <Button variant="destructive" size="sm" onClick={() => deleteStep.mutate(s.id)}>
                  Delete
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent className="py-2">
              <p className="text-xs text-muted-foreground">{s.description || "No description"}</p>
              <p className="text-xs font-mono mt-1">{s.script_path}</p>
            </CardContent>
          </Card>
        ))}
        {steps?.length === 0 && <p className="text-muted-foreground text-sm">No workflow steps registered</p>}
      </div>
    </div>
  );
}
