import { useState } from "react";
import { Link, useParams } from "@tanstack/react-router";
import { useProject } from "@/lib/queries/projects";
import { FileManager } from "@/components/file-manager";
import { RunsPanel } from "@/components/runs-panel";
import { Button } from "@/components/ui/button";

export function ProjectDetailPage() {
  const { projectId } = useParams({ strict: false }) as { projectId: string };
  const { data: project, isLoading } = useProject(projectId);
  const [tab, setTab] = useState<"files" | "runs">("files");

  if (isLoading) return <p>Loading...</p>;
  if (!project) return <p>Not found</p>;

  return (
    <div className="space-y-6">
      <div>
        <Link to="/projects" className="text-sm text-muted-foreground hover:underline">&larr; Projects</Link>
        <h1 className="text-2xl font-bold mt-1">{project.name}</h1>
        {project.description && <p className="text-muted-foreground">{project.description}</p>}
      </div>

      <div className="flex gap-2 border-b">
        <button
          className={`px-4 py-2 text-sm font-medium ${tab === "files" ? "border-b-2 border-primary" : "text-muted-foreground"}`}
          onClick={() => setTab("files")}
        >
          Files
        </button>
        <button
          className={`px-4 py-2 text-sm font-medium ${tab === "runs" ? "border-b-2 border-primary" : "text-muted-foreground"}`}
          onClick={() => setTab("runs")}
        >
          Runs
        </button>
      </div>

      {tab === "files" && <FileManager projectId={projectId} />}
      {tab === "runs" && <RunsPanel projectId={projectId} />}
    </div>
  );
}
