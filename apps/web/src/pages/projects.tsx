import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { useProjects, useCreateProject, useDeleteProject } from "@/lib/queries/projects";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

export function ProjectsPage() {
  const { data: projects, isLoading } = useProjects();
  const createProject = useCreateProject();
  const deleteProject = useDeleteProject();
  const [name, setName] = useState("");

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    await createProject.mutateAsync({ name: name.trim() });
    setName("");
  }

  if (isLoading) return <p>Loading...</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Projects</h1>

      <form onSubmit={handleCreate} className="flex gap-2">
        <Input
          placeholder="New project name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <Button type="submit" disabled={createProject.isPending}>Create</Button>
      </form>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {projects?.map((p) => (
          <Card key={p.id}>
            <CardHeader>
              <CardTitle>
                <Link to="/projects/$projectId" params={{ projectId: p.id }} className="hover:underline">
                  {p.name}
                </Link>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">{p.description || "No description"}</p>
              <div className="mt-4 flex justify-end">
                <Button variant="destructive" size="sm" onClick={() => deleteProject.mutate(p.id)}>
                  Delete
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
