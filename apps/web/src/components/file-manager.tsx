import { useState, useRef } from "react";
import { useFiles, useUploadFiles, useCreateDirectory, useDeleteFile } from "@/lib/queries/files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function FileManager({ projectId }: { projectId: string }) {
  const [parentId, setParentId] = useState<string | null>(null);
  const [breadcrumbs, setBreadcrumbs] = useState<{ id: string | null; name: string }[]>([
    { id: null, name: "Root" },
  ]);
  const [dirName, setDirName] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: files, isLoading } = useFiles(projectId, parentId);
  const uploadFiles = useUploadFiles(projectId);
  const createDir = useCreateDirectory(projectId);
  const deleteFile = useDeleteFile(projectId);

  function navigateToDir(id: string, name: string) {
    setParentId(id);
    setBreadcrumbs((prev) => [...prev, { id, name }]);
  }

  function navigateToBreadcrumb(index: number) {
    const crumb = breadcrumbs[index];
    setParentId(crumb.id);
    setBreadcrumbs((prev) => prev.slice(0, index + 1));
  }

  async function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    if (!e.target.files?.length) return;
    await uploadFiles.mutateAsync({ files: e.target.files, parentId: parentId ?? undefined });
    e.target.value = "";
  }

  async function handleCreateDir(e: React.FormEvent) {
    e.preventDefault();
    if (!dirName.trim()) return;
    await createDir.mutateAsync({ name: dirName.trim(), parent_id: parentId ?? undefined });
    setDirName("");
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 text-sm">
        {breadcrumbs.map((crumb, i) => (
          <span key={i}>
            {i > 0 && <span className="mx-1">/</span>}
            <button className="hover:underline" onClick={() => navigateToBreadcrumb(i)}>
              {crumb.name}
            </button>
          </span>
        ))}
      </div>

      <div className="flex gap-2">
        <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleUpload} />
        <Button size="sm" onClick={() => fileInputRef.current?.click()} disabled={uploadFiles.isPending}>
          Upload Files
        </Button>
        <form onSubmit={handleCreateDir} className="flex gap-1">
          <Input
            placeholder="New folder"
            value={dirName}
            onChange={(e) => setDirName(e.target.value)}
            className="h-8 w-40"
          />
          <Button size="sm" variant="outline" type="submit">Create</Button>
        </form>
      </div>

      {isLoading ? (
        <p>Loading...</p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left text-muted-foreground">
              <th className="py-2">Name</th>
              <th className="py-2 w-24">Size</th>
              <th className="py-2 w-32">Actions</th>
            </tr>
          </thead>
          <tbody>
            {files?.map((f) => (
              <tr key={f.id} className="border-b hover:bg-muted/50">
                <td className="py-2">
                  {f.is_directory ? (
                    <button className="font-medium hover:underline" onClick={() => navigateToDir(f.id, f.name)}>
                      📁 {f.name}
                    </button>
                  ) : (
                    <span>📄 {f.name}</span>
                  )}
                </td>
                <td className="py-2 text-muted-foreground">
                  {f.is_directory ? "—" : formatBytes(f.size_bytes)}
                </td>
                <td className="py-2 flex gap-2">
                  {!f.is_directory && (
                    <a
                      href={`/api/projects/${projectId}/files/${f.id}/download`}
                      className="text-blue-600 hover:underline text-xs"
                    >
                      Download
                    </a>
                  )}
                  <button
                    className="text-destructive hover:underline text-xs"
                    onClick={() => deleteFile.mutate(f.id)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
            {files?.length === 0 && (
              <tr><td colSpan={3} className="py-4 text-center text-muted-foreground">Empty</td></tr>
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
