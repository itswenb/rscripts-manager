import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  useMyFiles,
  usePublicFiles,
  useUploadMyFiles,
  useCreateMyDirectory,
  useDeleteFile,
  useRenameFile,
  useMoveFile,
  useMoveToPublic,
  UserFile,
} from "@/lib/queries/user-files";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Folder,
  File,
  Upload,
  Trash2,
  Download,
  FolderOpen,
  FileCode,
  FileSpreadsheet,
  FileImage,
  MoreHorizontal,
  Pencil,
  FolderInput,
  Globe,
  FolderPlus,
} from "lucide-react";
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

export function FileManagerPage() {
  const { t } = useTranslation();

  return (
    <div className="p-6 space-y-4">
      <h1 className="text-lg font-semibold">{t("files.title")}</h1>
      <Tabs defaultValue="my">
        <TabsList>
          <TabsTrigger value="my">{t("files.myFiles")}</TabsTrigger>
          <TabsTrigger value="public">{t("files.publicFiles")}</TabsTrigger>
        </TabsList>
        <TabsContent value="my">
          <MyFilesPanel />
        </TabsContent>
        <TabsContent value="public">
          <PublicFilesPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function MyFilesPanel() {
  const { t } = useTranslation();
  const [parentId, setParentId] = useState<string | null>(null);
  const [breadcrumb, setBreadcrumb] = useState<{ id: string | null; name: string }[]>([
    { id: null, name: t("files.root") },
  ]);
  const [dirDialogOpen, setDirDialogOpen] = useState(false);
  const [dirName, setDirName] = useState("");
  const [renameDialog, setRenameDialog] = useState<{ id: string; name: string } | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: files, isLoading } = useMyFiles(parentId);
  const uploadFiles = useUploadMyFiles();
  const createDir = useCreateMyDirectory();
  const deleteFile = useDeleteFile();
  const renameFile = useRenameFile();
  const moveFile = useMoveFile();
  const moveToPublic = useMoveToPublic();

  // Sort files: "scripts" folder always first, then other dirs, then files
  const sortedFiles = files?.slice().sort((a, b) => {
    if (a.name === "scripts" && a.is_directory) return -1;
    if (b.name === "scripts" && b.is_directory) return 1;
    if (a.is_directory && !b.is_directory) return -1;
    if (!a.is_directory && b.is_directory) return 1;
    return a.name.localeCompare(b.name);
  });

  // Auto-create "scripts" folder at root if it doesn't exist
  useEffect(() => {
    if (parentId === null && files && !files.some((f) => f.name === "scripts" && f.is_directory)) {
      createDir.mutate({ name: "scripts" });
    }
  }, [parentId, files]);

  function navigateTo(id: string | null, name: string) {
    setParentId(id);
    if (id === null) {
      setBreadcrumb([{ id: null, name: t("files.root") }]);
    } else {
      const idx = breadcrumb.findIndex((b) => b.id === id);
      if (idx >= 0) {
        setBreadcrumb(breadcrumb.slice(0, idx + 1));
      } else {
        setBreadcrumb([...breadcrumb, { id, name }]);
      }
    }
  }

  const scriptsFolder = files?.find((f) => f.name === "scripts" && f.is_directory);

  async function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    if (!e.target.files?.length) return;
    // Route .R files to scripts folder when at root
    const allFiles = Array.from(e.target.files);
    const rFiles = allFiles.filter((f) => /\.[rR]$/.test(f.name));
    const otherFiles = allFiles.filter((f) => !/\.[rR]$/.test(f.name));

    if (rFiles.length > 0 && parentId === null && scriptsFolder) {
      const dt = new DataTransfer();
      rFiles.forEach((f) => dt.items.add(f));
      await uploadFiles.mutateAsync({ files: dt.files, parentId: scriptsFolder.id });
    }
    if (otherFiles.length > 0) {
      const dt = new DataTransfer();
      otherFiles.forEach((f) => dt.items.add(f));
      await uploadFiles.mutateAsync({ files: dt.files, parentId: parentId ?? undefined });
    }
    if (rFiles.length > 0 && (parentId !== null || !scriptsFolder)) {
      const dt = new DataTransfer();
      rFiles.forEach((f) => dt.items.add(f));
      await uploadFiles.mutateAsync({ files: dt.files, parentId: parentId ?? undefined });
    }
    e.target.value = "";
  }

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setIsDraggingOver(false);
      if (e.dataTransfer.files.length > 0) {
        const input = document.createElement("input");
        input.type = "file";
        const dt = e.dataTransfer;
        Object.defineProperty(input, "files", { value: dt.files });
        await uploadFiles.mutateAsync({ files: dt.files, parentId: parentId ?? undefined });
      }
    },
    [parentId, uploadFiles]
  );

  function handleDragStart(e: React.DragEvent, file: UserFile) {
    e.dataTransfer.setData("application/x-file-id", file.id);
    e.dataTransfer.effectAllowed = "move";
  }

  function handleFolderDrop(e: React.DragEvent, targetFolderId: string) {
    e.preventDefault();
    e.stopPropagation();
    setDragOverId(null);
    const fileId = e.dataTransfer.getData("application/x-file-id");
    if (fileId && fileId !== targetFolderId) {
      moveFile.mutate({ id: fileId, parentId: targetFolderId });
    }
  }

  return (
    <div
      className={cn("mt-4 space-y-3", isDraggingOver && "ring-2 ring-primary/50 rounded-lg")}
      onDragOver={(e) => { e.preventDefault(); setIsDraggingOver(true); }}
      onDragLeave={() => setIsDraggingOver(false)}
      onDrop={handleDrop}
    >
      {/* Toolbar */}
      <div className="flex items-center gap-2">
        <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleUpload} />
        <Button size="sm" variant="outline" onClick={() => fileInputRef.current?.click()}>
          <Upload size={14} className="mr-1" /> {t("files.upload")}
        </Button>
        <Button size="sm" variant="outline" onClick={() => { setDirName(""); setDirDialogOpen(true); }}>
          <FolderPlus size={14} className="mr-1" /> {t("files.newFolder")}
        </Button>
      </div>

      {/* Breadcrumb */}
      <div className="flex items-center gap-1 text-sm text-muted-foreground">
        {breadcrumb.map((b, i) => (
          <span key={b.id ?? "root"} className="flex items-center gap-1">
            {i > 0 && <span>/</span>}
            <button
              className="hover:text-foreground hover:underline"
              onClick={() => navigateTo(b.id, b.name)}
            >
              {b.name}
            </button>
          </span>
        ))}
      </div>

      {isDraggingOver && (
        <div className="border-2 border-dashed border-primary/50 rounded-lg p-8 text-center text-muted-foreground">
          {t("files.dropHere")}
        </div>
      )}

      {/* File table */}
      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : !sortedFiles?.length ? (
        <p className="text-sm text-muted-foreground">{t("files.noFiles")}</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("files.fileName")}</TableHead>
              <TableHead className="w-20">{t("files.type")}</TableHead>
              <TableHead className="w-24">{t("files.size")}</TableHead>
              <TableHead className="w-32">{t("files.modified")}</TableHead>
              <TableHead className="w-20">{t("common.actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {sortedFiles.map((f) => {
              const Icon = f.is_directory ? Folder : getFileIcon(f.name);
              return (
                <TableRow
                  key={f.id}
                  draggable={!f.is_directory}
                  onDragStart={(e) => handleDragStart(e, f)}
                  onDragOver={(e) => {
                    if (f.is_directory) { e.preventDefault(); setDragOverId(f.id); }
                  }}
                  onDragLeave={() => setDragOverId(null)}
                  onDrop={(e) => f.is_directory && handleFolderDrop(e, f.id)}
                  className={cn(dragOverId === f.id && "bg-accent")}
                >
                  <TableCell>
                    {f.is_directory ? (
                      <button
                        className="flex items-center gap-2 font-medium hover:underline"
                        onClick={() => navigateTo(f.id, f.name)}
                      >
                        <Icon size={14} className="text-muted-foreground" />
                        {f.name}
                      </button>
                    ) : (
                      <span className="flex items-center gap-2">
                        <Icon size={14} className="text-muted-foreground" />
                        {f.name}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {f.is_directory ? t("files.folder") : f.name.split(".").pop()?.toUpperCase()}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {f.is_directory ? "—" : formatBytes(f.size_bytes)}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {new Date(f.created_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    <FileActions
                      file={f}
                      onRename={() => setRenameDialog({ id: f.id, name: f.name })}
                      onDelete={() => deleteFile.mutate(f.id)}
                      onMoveToPublic={() => moveToPublic.mutate(f.id)}
                    />
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      )}

      {/* New folder dialog */}
      <Dialog open={dirDialogOpen} onOpenChange={setDirDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("files.newFolder")}</DialogTitle>
          </DialogHeader>
          <Input
            value={dirName}
            onChange={(e) => setDirName(e.target.value)}
            placeholder={t("common.name")}
            onKeyDown={(e) => {
              if (e.key === "Enter" && dirName.trim()) {
                createDir.mutate({ name: dirName, parent_id: parentId ?? undefined });
                setDirDialogOpen(false);
              }
            }}
          />
          <DialogFooter>
            <Button
              onClick={() => {
                if (dirName.trim()) {
                  createDir.mutate({ name: dirName, parent_id: parentId ?? undefined });
                  setDirDialogOpen(false);
                }
              }}
            >
              {t("common.create")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Rename dialog */}
      <Dialog open={!!renameDialog} onOpenChange={() => setRenameDialog(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("files.rename")}</DialogTitle>
          </DialogHeader>
          <Input
            value={renameDialog?.name ?? ""}
            onChange={(e) => renameDialog && setRenameDialog({ ...renameDialog, name: e.target.value })}
            onKeyDown={(e) => {
              if (e.key === "Enter" && renameDialog?.name.trim()) {
                renameFile.mutate({ id: renameDialog.id, name: renameDialog.name });
                setRenameDialog(null);
              }
            }}
          />
          <DialogFooter>
            <Button
              onClick={() => {
                if (renameDialog?.name.trim()) {
                  renameFile.mutate({ id: renameDialog.id, name: renameDialog.name });
                  setRenameDialog(null);
                }
              }}
            >
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function PublicFilesPanel() {
  const { t } = useTranslation();
  const [parentId, setParentId] = useState<string | null>(null);
  const { data: files, isLoading } = usePublicFiles(parentId);

  const sortedFiles = files?.slice().sort((a, b) => {
    if (a.name === "scripts" && a.is_directory) return -1;
    if (b.name === "scripts" && b.is_directory) return 1;
    if (a.is_directory && !b.is_directory) return -1;
    if (!a.is_directory && b.is_directory) return 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <div className="mt-4 space-y-3">
      {parentId && (
        <Button size="sm" variant="ghost" onClick={() => setParentId(null)}>
          {t("common.back")}
        </Button>
      )}
      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : !sortedFiles?.length ? (
        <p className="text-sm text-muted-foreground">{t("files.noFiles")}</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("files.fileName")}</TableHead>
              <TableHead className="w-20">{t("files.type")}</TableHead>
              <TableHead className="w-24">{t("files.size")}</TableHead>
              <TableHead className="w-32">{t("files.modified")}</TableHead>
              <TableHead className="w-16">{t("common.actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {sortedFiles.map((f) => {
              const Icon = f.is_directory ? Folder : getFileIcon(f.name);
              return (
                <TableRow key={f.id}>
                  <TableCell>
                    {f.is_directory ? (
                      <button
                        className="flex items-center gap-2 font-medium hover:underline"
                        onClick={() => setParentId(f.id)}
                      >
                        <Icon size={14} className="text-muted-foreground" />
                        {f.name}
                      </button>
                    ) : (
                      <span className="flex items-center gap-2">
                        <Icon size={14} className="text-muted-foreground" />
                        {f.name}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {f.is_directory ? t("files.folder") : f.name.split(".").pop()?.toUpperCase()}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {f.is_directory ? "—" : formatBytes(f.size_bytes)}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {new Date(f.created_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    {!f.is_directory && (
                      <a
                        href={`/api/files/${f.id}/download`}
                        className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground"
                      >
                        <Download size={13} />
                      </a>
                    )}
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      )}
    </div>
  );
}

function FileActions({
  file,
  onRename,
  onDelete,
  onMoveToPublic,
}: {
  file: UserFile;
  onRename: () => void;
  onDelete: () => void;
  onMoveToPublic: () => void;
}) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground">
          <MoreHorizontal size={14} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {!file.is_directory && (
          <DropdownMenuItem onClick={() => window.open(`/api/files/${file.id}/download`)}>
            <Download size={14} className="mr-2" /> {t("files.download")}
          </DropdownMenuItem>
        )}
        <DropdownMenuItem onClick={onRename}>
          <Pencil size={14} className="mr-2" /> {t("files.rename")}
        </DropdownMenuItem>
        {!file.is_public && (
          <DropdownMenuItem onClick={onMoveToPublic}>
            <Globe size={14} className="mr-2" /> {t("files.moveToPublic")}
          </DropdownMenuItem>
        )}
        <DropdownMenuItem onClick={onDelete} className="text-destructive">
          <Trash2 size={14} className="mr-2" /> {t("common.delete")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
