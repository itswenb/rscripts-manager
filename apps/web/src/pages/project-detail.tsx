import { Link, useParams } from "@tanstack/react-router";
import { useProject } from "@/lib/queries/projects";
import { FileManager } from "@/components/file-manager";
import { RunsPanel } from "@/components/runs-panel";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { ChevronRight } from "lucide-react";

export function ProjectDetailPage() {
  const { projectId } = useParams({ strict: false }) as { projectId: string };
  const { data: project, isLoading } = useProject(projectId);

  if (isLoading) return <p className="text-sm text-muted-foreground">Loading...</p>;
  if (!project) return <p className="text-sm text-muted-foreground">Not found</p>;

  return (
    <div className="space-y-4">
      <div>
        <div className="flex items-center gap-1 text-sm text-muted-foreground mb-1">
          <Link to="/projects" className="hover:text-foreground transition-colors">Projects</Link>
          <ChevronRight size={13} />
          <span className="text-foreground">{project.name}</span>
        </div>
        {project.description && <p className="text-sm text-muted-foreground">{project.description}</p>}
      </div>

      <Tabs defaultValue="files">
        <TabsList>
          <TabsTrigger value="files">Files</TabsTrigger>
          <TabsTrigger value="runs">Runs</TabsTrigger>
        </TabsList>
        <TabsContent value="files">
          <FileManager projectId={projectId} />
        </TabsContent>
        <TabsContent value="runs">
          <RunsPanel projectId={projectId} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
