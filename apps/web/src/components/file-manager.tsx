import { useState, useRef } from "react";
import { useFiles, useUploadFiles, useCreateDirectory, useDeleteFile } from "@/lib/queries/files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Folder, File, Upload, Trash2, Download, ChevronRight } from "lucide-react";

export function FileManager({ projectId }: { projectId: string }) {
  const [parentId, setParentId] = useState<string | null>(null);
  const [breadcrumbs, setBreadcrumbs] = useState<{ id: string | null; name: string }[]>([{ id: null, name: "Root" }]);
  const [dirDialogOpen, setDirDialogOpen] = useState(false);
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
    setDirDialogOpen(false);
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1 text-sm text-muted-foreground">
          {breadcrumbs.map((crumb, i) => (
            <span key={i} className="flex items-center gap-1">
              {i > 0 && <ChevronRight size={12} />}
              <button
                className={i === breadcrumbs.length - 1 ? "text-foreground font-medium" : "hover:text-foreground transition-colors"}
                onClick={() => navigateToBreadcrumb(i)}
              >
                {crumb.name}
              </button>
            </span>
          ))}
        </div>
        <div className="flex gap-2">
          <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleUpload} />
          <Button size="sm" variant="outline" onClick={() => fileInputRef.current?.click()} disabled={uploadFiles.isPending}>
            <Upload size={13} className="mr-1.5" />
            Upload
          </Button>
          <Button size="sm" variant="outline" onClick={() => setDirDialogOpen(true)}>
            <Folder size={13} className="mr-1.5" />
            New Folder
          </Button>
        </div>
      </div>

      <Dialog open={dirDialogOpen} onOpenChange={setDirDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>New Folder</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreateDir} className="space-y-4 pt-2">
            <div className="space-y-1.5">
              <Label htmlFor="dir-name">Folder Name</Label>
              <Input id="dir-name" value={dirName} onChange={(e) => setDirName(e.target.value)} autoFocus />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setDirDialogOpen(false)}>Cancel</Button>
              <Button type="submit" disabled={!dirName.trim()}>Create</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {isLoading ? (
        <p className="text-sm text-muted-foreground">Loading...</p>
      ) : (
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead className="w-24">Size</TableHead>
                <TableHead className="w-20"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {files?.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-center text-muted-foreground py-8">Empty</TableCell>
                </TableRow>
              )}
              {files?.map((f) => (
                <TableRow key={f.id}>
                  <TableCell>
                    {f.is_directory ? (
                      <button
                        className="flex items-center gap-2 font-medium hover:underline"
                        onClick={() => navigateToDir(f.id, f.name)}
                      >
                        <Folder size={14} className="text-muted-foreground" />
                        {f.name}
                      </button>
                    ) : (
                      <span className="flex items-center gap-2">
                        <File size={14} className="text-muted-foreground" />
                        {f.name}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {f.is_directory ? "—" : formatBytes(f.size_bytes)}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      {!f.is_directory && (
                        <a
                          href={`/api/projects/${projectId}/files/${f.id}/download`}
                          className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
                          title="Download"
                        >
                          <Download size={13} />
                        </a>
                      )}
                      <button
                        className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
                        onClick={() => deleteFile.mutate(f.id)}
                        title="Delete"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
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
