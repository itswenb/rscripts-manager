# Frontend Feature Pages Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement all feature pages — Projects list/detail, File Manager, Workflow Steps admin, and Script Runs monitor.

**Architecture:** Each feature is a route under `/_authenticated/`. TanStack Query handles data fetching with query keys matching the API structure. Pages use shadcn/ui components. File manager uses a tree/list view with upload dropzone. Runs page shows status with polling for active runs.

**Tech Stack:** @tanstack/react-query, @tanstack/react-router, TailwindCSS, shadcn/ui, lucide-react

**Prerequisite:** Frontend Foundation plan must be completed first.

---

### Task 1: Projects list page

**Files:**

- Create: `apps/web/src/routes/_authenticated/projects/index.tsx`
- Create: `apps/web/src/lib/queries/projects.ts`

**Step 1: Create query hooks**

`apps/web/src/lib/queries/projects.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface Project {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export function useProjects() {
  return useQuery({
    queryKey: ["projects"],
    queryFn: () => api.get<Project[]>("/projects"),
  });
}

export function useCreateProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; description?: string }) =>
      api.post<Project>("/projects", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["projects"] }),
  });
}

export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/projects/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["projects"] }),
  });
}
```

**Step 2: Create projects list page**

`apps/web/src/routes/_authenticated/projects/index.tsx`:

```tsx
import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useProjects, useCreateProject, useDeleteProject } from "@/lib/queries/projects";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";

export const Route = createFileRoute("/_authenticated/projects/")({
  component: ProjectsPage,
});

function ProjectsPage() {
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
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Projects</h1>
      </div>

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
              <p className="text-sm text-gray-500">{p.description || "No description"}</p>
              <div className="mt-4 flex justify-end">
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => deleteProject.mutate(p.id)}
                >
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
```

**Step 3: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 4: Commit**

```bash
git add apps/web/src/lib/queries/projects.ts apps/web/src/routes/_authenticated/projects/
git commit -m "feat(web): add projects list page"
```

---

### Task 2: Project detail page with tabs

**Files:**

- Create: `apps/web/src/routes/_authenticated/projects/$projectId.tsx`

**Step 1: Create project detail page**

`apps/web/src/routes/_authenticated/projects/$projectId.tsx`:

```tsx
import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import type { Project } from "@/lib/queries/projects";

export const Route = createFileRoute("/_authenticated/projects/$projectId")({
  component: ProjectDetailPage,
});

function ProjectDetailPage() {
  const { projectId } = Route.useParams();
  const [tab, setTab] = useState<"files" | "runs">("files");

  const { data: project, isLoading } = useQuery({
    queryKey: ["projects", projectId],
    queryFn: () => api.get<Project>(`/projects/${projectId}`),
  });

  if (isLoading) return <p>Loading...</p>;
  if (!project) return <p>Not found</p>;

  return (
    <div className="space-y-6">
      <div>
        <Link to="/projects" className="text-sm text-gray-500 hover:underline">&larr; Projects</Link>
        <h1 className="text-2xl font-bold mt-1">{project.name}</h1>
        <p className="text-gray-500">{project.description}</p>
      </div>

      <div className="flex gap-2 border-b">
        <button
          className={`px-4 py-2 text-sm font-medium ${tab === "files" ? "border-b-2 border-primary" : "text-gray-500"}`}
          onClick={() => setTab("files")}
        >
          Files
        </button>
        <button
          className={`px-4 py-2 text-sm font-medium ${tab === "runs" ? "border-b-2 border-primary" : "text-gray-500"}`}
          onClick={() => setTab("runs")}
        >
          Runs
        </button>
      </div>

      {tab === "files" && <FilesTab projectId={projectId} />}
      {tab === "runs" && <RunsTab projectId={projectId} />}
    </div>
  );
}

function FilesTab({ projectId }: { projectId: string }) {
  return <p className="text-gray-500">File manager — implemented in Task 3</p>;
}

function RunsTab({ projectId }: { projectId: string }) {
  return <p className="text-gray-500">Runs list — implemented in Task 5</p>;
}
```

**Step 2: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 3: Commit**

```bash
git add apps/web/src/routes/_authenticated/projects/\$projectId.tsx
git commit -m "feat(web): add project detail page with tabs"
```

---

### Task 3: File Manager component

**Files:**

- Create: `apps/web/src/lib/queries/files.ts`
- Create: `apps/web/src/components/file-manager.tsx`
- Modify: `apps/web/src/routes/_authenticated/projects/$projectId.tsx`

**Step 1: Create file query hooks**

`apps/web/src/lib/queries/files.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { getToken } from "@/lib/auth";

export interface FileAsset {
  id: string;
  project_id: string;
  parent_id: string | null;
  name: string;
  is_directory: boolean;
  size_bytes: number;
  mime_type: string | null;
  created_at: string;
}

export function useFiles(projectId: string, parentId?: string | null) {
  const query = parentId ? `?parent_id=${parentId}` : "";
  return useQuery({
    queryKey: ["files", projectId, parentId ?? "root"],
    queryFn: () => api.get<FileAsset[]>(`/projects/${projectId}/files${query}`),
  });
}

export function useUploadFiles(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ files, parentId }: { files: FileList; parentId?: string }) => {
      const form = new FormData();
      for (const f of files) form.append("files", f);
      const query = parentId ? `?parent_id=${parentId}` : "";
      return api.post<FileAsset[]>(`/projects/${projectId}/files${query}`, form);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}

export function useCreateDirectory(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; parent_id?: string }) =>
      api.post<FileAsset>(`/projects/${projectId}/files/directory`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}

export function useDeleteFile(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (assetId: string) =>
      api.delete(`/projects/${projectId}/files/${assetId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files", projectId] }),
  });
}

export function downloadUrl(projectId: string, assetId: string): string {
  return `/api/projects/${projectId}/files/${assetId}/download`;
}
```

**Step 2: Create FileManager component**

`apps/web/src/components/file-manager.tsx`:

```tsx
import { useState, useRef } from "react";
import { useFiles, useUploadFiles, useCreateDirectory, useDeleteFile, downloadUrl } from "@/lib/queries/files";
import type { FileAsset } from "@/lib/queries/files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface Props {
  projectId: string;
}

export function FileManager({ projectId }: Props) {
  const [currentDir, setCurrentDir] = useState<string | null>(null);
  const [breadcrumb, setBreadcrumb] = useState<{ id: string | null; name: string }[]>([
    { id: null, name: "Root" },
  ]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [newDirName, setNewDirName] = useState("");

  const { data: files, isLoading } = useFiles(projectId, currentDir);
  const upload = useUploadFiles(projectId);
  const createDir = useCreateDirectory(projectId);
  const deleteFile = useDeleteFile(projectId);

  function navigateToDir(dir: FileAsset) {
    setCurrentDir(dir.id);
    setBreadcrumb((prev) => [...prev, { id: dir.id, name: dir.name }]);
  }

  function navigateToBreadcrumb(index: number) {
    const target = breadcrumb[index];
    setCurrentDir(target.id);
    setBreadcrumb((prev) => prev.slice(0, index + 1));
  }

  function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    if (e.target.files?.length) {
      upload.mutate({ files: e.target.files, parentId: currentDir ?? undefined });
    }
  }

  function handleCreateDir(e: React.FormEvent) {
    e.preventDefault();
    if (!newDirName.trim()) return;
    createDir.mutate({ name: newDirName.trim(), parent_id: currentDir ?? undefined });
    setNewDirName("");
  }

  return (
    <div className="space-y-4">
      {/* Breadcrumb */}
      <div className="flex items-center gap-1 text-sm">
        {breadcrumb.map((b, i) => (
          <span key={i}>
            {i > 0 && <span className="mx-1">/</span>}
            <button className="hover:underline" onClick={() => navigateToBreadcrumb(i)}>
              {b.name}
            </button>
          </span>
        ))}
      </div>

      {/* Actions */}
      <div className="flex gap-2">
        <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleUpload} />
        <Button size="sm" onClick={() => fileInputRef.current?.click()}>Upload</Button>
        <form onSubmit={handleCreateDir} className="flex gap-1">
          <Input
            placeholder="New folder"
            value={newDirName}
            onChange={(e) => setNewDirName(e.target.value)}
            className="h-8 w-40"
          />
          <Button size="sm" variant="outline" type="submit">Create</Button>
        </form>
      </div>

      {/* File list */}
      {isLoading ? (
        <p>Loading...</p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left">
              <th className="py-2">Name</th>
              <th className="py-2 w-24">Size</th>
              <th className="py-2 w-32">Actions</th>
            </tr>
          </thead>
          <tbody>
            {files?.map((f) => (
              <tr key={f.id} className="border-b hover:bg-gray-50">
                <td className="py-2">
                  {f.is_directory ? (
                    <button className="font-medium hover:underline" onClick={() => navigateToDir(f)}>
                      📁 {f.name}
                    </button>
                  ) : (
                    <span>📄 {f.name}</span>
                  )}
                </td>
                <td className="py-2 text-gray-500">
                  {f.is_directory ? "—" : formatBytes(f.size_bytes)}
                </td>
                <td className="py-2 flex gap-1">
                  {!f.is_directory && (
                    <a href={downloadUrl(projectId, f.id)} className="text-blue-600 hover:underline text-xs">
                      Download
                    </a>
                  )}
                  <button
                    className="text-red-600 hover:underline text-xs"
                    onClick={() => deleteFile.mutate(f.id)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
            {files?.length === 0 && (
              <tr><td colSpan={3} className="py-4 text-center text-gray-400">Empty</td></tr>
            )}
          </tbody>
        </table>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
```

**Step 3: Wire into project detail page**

In `$projectId.tsx`, replace the `FilesTab` placeholder:

```tsx
import { FileManager } from "@/components/file-manager";

function FilesTab({ projectId }: { projectId: string }) {
  return <FileManager projectId={projectId} />;
}
```

**Step 4: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/web/src/lib/queries/files.ts apps/web/src/components/file-manager.tsx apps/web/src/routes/_authenticated/projects/\$projectId.tsx
git commit -m "feat(web): add file manager component with upload/browse/delete"
```

---

### Task 4: Workflow Steps admin page

**Files:**

- Create: `apps/web/src/lib/queries/workflow-steps.ts`
- Create: `apps/web/src/routes/_authenticated/workflow-steps.tsx`

**Step 1: Create query hooks**

`apps/web/src/lib/queries/workflow-steps.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface WorkflowStep {
  id: string;
  name: string;
  description: string;
  script_path: string;
  input_schema: unknown[];
  param_schema: unknown[];
  output_dir_name: string;
  created_at: string;
}

export function useWorkflowSteps() {
  return useQuery({
    queryKey: ["workflow-steps"],
    queryFn: () => api.get<WorkflowStep[]>("/workflow-steps"),
  });
}

export function useCreateWorkflowStep() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; script_path: string; description?: string }) =>
      api.post<WorkflowStep>("/workflow-steps", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workflow-steps"] }),
  });
}

export function useDeleteWorkflowStep() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/workflow-steps/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workflow-steps"] }),
  });
}
```

**Step 2: Create workflow steps page**

`apps/web/src/routes/_authenticated/workflow-steps.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useWorkflowSteps, useCreateWorkflowStep, useDeleteWorkflowStep } from "@/lib/queries/workflow-steps";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

export const Route = createFileRoute("/_authenticated/workflow-steps")({
  component: WorkflowStepsPage,
});

function WorkflowStepsPage() {
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
            <CardHeader className="pb-2">
              <CardTitle className="text-base">{s.name}</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-gray-500 font-mono">{s.script_path}</p>
              <p className="text-sm text-gray-400 mt-1">{s.description || "No description"}</p>
              <div className="mt-3 flex justify-end">
                <Button variant="destructive" size="sm" onClick={() => deleteStep.mutate(s.id)}>
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
```

**Step 3: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 4: Commit**

```bash
git add apps/web/src/lib/queries/workflow-steps.ts apps/web/src/routes/_authenticated/workflow-steps.tsx
git commit -m "feat(web): add workflow steps admin page"
```

---

### Task 5: Script Runs page

**Files:**

- Create: `apps/web/src/lib/queries/runs.ts`
- Modify: `apps/web/src/routes/_authenticated/projects/$projectId.tsx`

**Step 1: Create run query hooks**

`apps/web/src/lib/queries/runs.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface ScriptRun {
  id: string;
  project_id: string;
  workflow_step_id: string;
  status: "pending" | "running" | "completed" | "failed";
  inputs: Record<string, unknown>;
  params: Record<string, unknown>;
  stdout: string | null;
  stderr: string | null;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
}

export interface OutputFile {
  id: string;
  run_id: string;
  name: string;
  size_bytes: number;
  mime_type: string | null;
}

export function useRuns(projectId: string) {
  return useQuery({
    queryKey: ["runs", projectId],
    queryFn: () => api.get<ScriptRun[]>(`/projects/${projectId}/runs`),
    refetchInterval: (query) => {
      const runs = query.state.data;
      const hasActive = runs?.some((r) => r.status === "pending" || r.status === "running");
      return hasActive ? 3000 : false;
    },
  });
}

export function useCreateRun(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { workflow_step_id: string; inputs?: object; params?: object }) =>
      api.post<ScriptRun>(`/projects/${projectId}/runs`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["runs", projectId] }),
  });
}

export function useRunOutputs(projectId: string, runId: string) {
  return useQuery({
    queryKey: ["runs", projectId, runId, "outputs"],
    queryFn: () => api.get<OutputFile[]>(`/projects/${projectId}/runs/${runId}/outputs`),
  });
}
```

**Step 2: Implement RunsTab in project detail**

Replace the `RunsTab` placeholder in `$projectId.tsx`:

```tsx
import { useRuns, useCreateRun } from "@/lib/queries/runs";
import { useWorkflowSteps } from "@/lib/queries/workflow-steps";

function RunsTab({ projectId }: { projectId: string }) {
  const { data: runs, isLoading } = useRuns(projectId);
  const { data: steps } = useWorkflowSteps();
  const createRun = useCreateRun(projectId);
  const [selectedStep, setSelectedStep] = useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedStep) return;
    createRun.mutate({ workflow_step_id: selectedStep });
  }

  return (
    <div className="space-y-4">
      <form onSubmit={handleSubmit} className="flex gap-2">
        <select
          value={selectedStep}
          onChange={(e) => setSelectedStep(e.target.value)}
          className="border rounded-md px-3 py-2 text-sm"
        >
          <option value="">Select workflow step...</option>
          {steps?.map((s) => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>
        <Button type="submit" disabled={createRun.isPending}>Run</Button>
      </form>

      {isLoading ? (
        <p>Loading...</p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left">
              <th className="py-2">Status</th>
              <th className="py-2">Step</th>
              <th className="py-2">Created</th>
              <th className="py-2">Duration</th>
            </tr>
          </thead>
          <tbody>
            {runs?.map((r) => (
              <tr key={r.id} className="border-b hover:bg-gray-50">
                <td className="py-2">
                  <StatusBadge status={r.status} />
                </td>
                <td className="py-2">
                  {steps?.find((s) => s.id === r.workflow_step_id)?.name ?? r.workflow_step_id.slice(0, 8)}
                </td>
                <td className="py-2 text-gray-500">
                  {new Date(r.created_at).toLocaleString()}
                </td>
                <td className="py-2 text-gray-500">
                  {r.started_at && r.finished_at
                    ? `${((new Date(r.finished_at).getTime() - new Date(r.started_at).getTime()) / 1000).toFixed(1)}s`
                    : "—"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    pending: "bg-yellow-100 text-yellow-800",
    running: "bg-blue-100 text-blue-800",
    completed: "bg-green-100 text-green-800",
    failed: "bg-red-100 text-red-800",
  };
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-medium ${colors[status] ?? "bg-gray-100"}`}>
      {status}
    </span>
  );
}
```

**Step 3: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 4: Commit**

```bash
git add apps/web/src/lib/queries/runs.ts apps/web/src/routes/_authenticated/projects/\$projectId.tsx
git commit -m "feat(web): add script runs tab with status polling"
```

---

### Task 6: Update sidebar navigation

**Files:**

- Modify: `apps/web/src/routes/_authenticated.tsx`

**Step 1: Add navigation links**

In the `AuthenticatedLayout` sidebar, ensure these links exist:

```tsx
<nav className="space-y-1">
  <Link to="/projects" className="block px-3 py-2 rounded hover:bg-gray-100 text-sm">
    Projects
  </Link>
  <Link to="/workflow-steps" className="block px-3 py-2 rounded hover:bg-gray-100 text-sm">
    Workflow Steps
  </Link>
</nav>
```

**Step 2: Update router to include all routes**

In `apps/web/src/router.ts`, import and register all route files in the route tree.

**Step 3: Verify**

Run: `cd apps/web && pnpm dev`
Expected: Navigate between pages, all routes resolve

**Step 4: Commit**

```bash
git add apps/web/src/routes/ apps/web/src/router.ts
git commit -m "feat(web): wire up sidebar navigation for all pages"
```

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-2 | Projects list + detail shell |
| 2 | 3 | File manager component |
| 3 | 4-5 | Workflow steps + runs |
| 4 | 6 | Navigation wiring |

## Notes

- Runs page uses `refetchInterval` that auto-polls every 3s when there are active (pending/running) runs.
- File manager uses breadcrumb navigation for directory traversal.
- Download links point directly to the API endpoint (browser handles the download via Content-Disposition header).
- All pages require authentication via the `_authenticated` layout route guard.
- The route tree must be manually assembled since Farm doesn't have a TanStack Router code-gen plugin.
