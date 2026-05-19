import { useState, useRef } from "react";
import { useParams } from "@tanstack/react-router";
import { useFiles, useUploadFiles, useCreateDirectory, useDeleteFile, FileAsset } from "@/lib/queries/files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Folder, File, Upload, Trash2, Download, ChevronRight, FolderOpen, FileCode, FileSpreadsheet, FileImage } from "lucide-react";
import { cn } from "@/lib/utils";

function getFileIcon(name: string) {
  const ext = name.split(".").pop()?.toLowerCase();
  if (["py", "r", "js", "ts", "sh"].includes(ext ?? "")) return FileCode;
  if (["csv", "tsv", "xlsx", "json"].includes(ext ?? "")) return FileSpreadsheet;
  if (["png", "jpg", "jpeg", "gif", "svg"].includes(ext ?? "")) return FileImage;
  return File;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function DirectoryTree({ projectId, selectedId, onSelect }: {
  projectId: string;
  selectedId: string | null;
  onSelect: (id: string | null, name: string) => void;
}) {
  const { data: rootFiles } = useFiles(projectId, null);
  const directories = rootFiles?.filter(f => f.is_directory) ?? [];

  return (
    <div className="space-y-0.5">
      <button
        onClick={() => onSelect(null, "Root")}
        className={cn(
          "flex items-center gap-2 px-2 py-1.5 rounded text-sm w-full text-left",
          selectedId === null ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50"
        )}
      >
        <FolderOpen size={14} /> Root
      </button>
      {directories.map(dir => (
        <button
          key={dir.id}
          onClick={() => onSelect(dir.id, dir.name)}
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded text-sm w-full text-left pl-5",
            selectedId === dir.id ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50"
          )}
        >
          <Folder size={14} /> {dir.name}
        </button>
      ))}
    </div>
  );
}

export function ProjectExplorerPage() {
  const { projectId } = useParams({ strict: false }) as { projectId: string };
  const [selectedDir, setSelectedDir] = useState<{ id: string | null; name: string }>({ id: null, name: "Root" });
  const [dirDialogOpen, setDirDialogOpen] = useState(false);
  const [dirName, setDirName] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: files, isLoading } = useFiles(projectId, selectedDir.id);
  const uploadFiles = useUploadFiles(projectId);
  const createDir = useCreateDirectory(projectId);
  const deleteFile = useDeleteFile(projectId);

  async function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    if (!e.target.files?.length) return;
    await uploadFiles.mutateAsync({ files: e.target.files, parentId: selectedDir.id ?? undefined });
    e.target.value = "";
  }

  async function handleCreateDir(e: React.FormEvent) {
    e.preventDefault();
    if (!dirName.trim()) return;
    await createDir.mutateAsync({ name: dirName.trim(), parent_id: selectedDir.id ?? undefined });
    setDirName("");
    setDirDialogOpen(false);
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)]">
      <aside className="w-56 border-r p-3 overflow-auto shrink-0">
        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2 px-2">Directories</p>
        <DirectoryTree projectId={projectId} selectedId={selectedDir.id} onSelect={(id, name) => setSelectedDir({ id, name })} />
      </aside>
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="flex items-center justify-between px-4 py-3 border-b">
          <div className="flex items-center gap-1 text-sm text-muted-foreground">
            <button onClick={() => setSelectedDir({ id: null, name: "Root" })} className="hover:text-foreground">Root</button>
            {selectedDir.id && (
              <>
                <ChevronRight size={12} />
                <span className="text-foreground font-medium">{selectedDir.name}</span>
              </>
            )}
          </div>
          <div className="flex gap-2">
            <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleUpload} />
            <Button size="sm" variant="outline" onClick={() => fileInputRef.current?.click()} disabled={uploadFiles.isPending}>
              <Upload size={13} className="mr-1.5" />Upload
            </Button>
            <Button size="sm" variant="outline" onClick={() => setDirDialogOpen(true)}>
              <Folder size={13} className="mr-1.5" />New Folder
            </Button>
          </div>
        </div>

        <Dialog open={dirDialogOpen} onOpenChange={setDirDialogOpen}>
          <DialogContent className="max-w-sm">
            <DialogHeader><DialogTitle>New Folder</DialogTitle></DialogHeader>
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

        <div className="flex-1 overflow-auto">
          {isLoading ? (
            <p className="p-4 text-sm text-muted-foreground">Loading...</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead className="w-24">Type</TableHead>
                  <TableHead className="w-24">Size</TableHead>
                  <TableHead className="w-36">Modified</TableHead>
                  <TableHead className="w-20"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {files?.length === 0 && (
                  <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">Empty directory</TableCell></TableRow>
                )}
                {files?.map(f => {
                  const Icon = f.is_directory ? Folder : getFileIcon(f.name);
                  const ext = f.name.split(".").pop()?.toUpperCase();
                  return (
                    <TableRow key={f.id}>
                      <TableCell>
                        {f.is_directory ? (
                          <button className="flex items-center gap-2 font-medium hover:underline" onClick={() => setSelectedDir({ id: f.id, name: f.name })}>
                            <Icon size={14} className="text-muted-foreground" />{f.name}
                          </button>
                        ) : (
                          <span className="flex items-center gap-2"><Icon size={14} className="text-muted-foreground" />{f.name}</span>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground text-xs">{f.is_directory ? "Folder" : ext}</TableCell>
                      <TableCell className="text-muted-foreground text-xs">{f.is_directory ? "—" : formatBytes(f.size_bytes)}</TableCell>
                      <TableCell className="text-muted-foreground text-xs">{new Date(f.created_at).toLocaleDateString()}</TableCell>
                      <TableCell>
                        <div className="flex items-center gap-1">
                          {!f.is_directory && (
                            <a href={`/api/projects/${projectId}/files/${f.id}/download`} className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors" title="Download">
                              <Download size={13} />
                            </a>
                          )}
                          <button className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors" onClick={() => deleteFile.mutate(f.id)} title="Delete">
                            <Trash2 size={13} />
                          </button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </div>
      </div>
    </div>
  );
}
