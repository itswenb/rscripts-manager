import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useUsers, useCreateUser, useUpdateUser, useDeleteUser, User } from "@/lib/queries/users";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Trash2, UserPlus, KeyRound } from "lucide-react";

export function UsersPage() {
  const { t } = useTranslation();
  const { data: users, isLoading } = useUsers();
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const deleteUser = useDeleteUser();

  const [createOpen, setCreateOpen] = useState(false);
  const [form, setForm] = useState({ username: "", password: "", role: "viewer" });
  const [resetPwDialog, setResetPwDialog] = useState<User | null>(null);
  const [newPassword, setNewPassword] = useState("");

  async function handleCreate() {
    if (!form.username || !form.password) return;
    await createUser.mutateAsync(form);
    setCreateOpen(false);
    setForm({ username: "", password: "", role: "viewer" });
  }

  async function handleResetPassword() {
    if (!resetPwDialog || !newPassword) return;
    await updateUser.mutateAsync({ id: resetPwDialog.id, password: newPassword });
    setResetPwDialog(null);
    setNewPassword("");
  }

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">{t("users.title")}</h1>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <UserPlus size={14} className="mr-1" /> {t("users.addUser")}
        </Button>
      </div>

      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("auth.username")}</TableHead>
              <TableHead>{t("users.role")}</TableHead>
              <TableHead>{t("common.createdAt")}</TableHead>
              <TableHead className="w-32">{t("common.actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users?.map((u) => (
              <TableRow key={u.id}>
                <TableCell className="font-medium">{u.username}</TableCell>
                <TableCell>
                  <Badge variant={u.role === "admin" ? "default" : "secondary"}>
                    {u.role === "admin" ? t("users.admin") : t("users.user")}
                  </Badge>
                </TableCell>
                <TableCell className="text-muted-foreground text-xs">
                  {new Date(u.created_at).toLocaleDateString()}
                </TableCell>
                <TableCell>
                  <div className="flex items-center gap-1">
                    <button
                      className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground"
                      onClick={() => { setResetPwDialog(u); setNewPassword(""); }}
                      title={t("users.resetPassword")}
                    >
                      <KeyRound size={13} />
                    </button>
                    <button
                      className="inline-flex items-center justify-center h-7 w-7 rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive"
                      onClick={() => deleteUser.mutate(u.id)}
                      title={t("common.delete")}
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      {/* Create User Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.addUser")}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div>
              <Label>{t("auth.username")}</Label>
              <Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
            </div>
            <div>
              <Label>{t("auth.password")}</Label>
              <Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
            </div>
            <div>
              <Label>{t("users.role")}</Label>
              <select
                className="w-full border rounded-md px-3 py-2 text-sm bg-background"
                value={form.role}
                onChange={(e) => setForm({ ...form, role: e.target.value })}
              >
                <option value="viewer">{t("users.user")}</option>
                <option value="admin">{t("users.admin")}</option>
              </select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleCreate}>{t("common.create")}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Reset Password Dialog */}
      <Dialog open={!!resetPwDialog} onOpenChange={() => setResetPwDialog(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.resetPassword")} - {resetPwDialog?.username}</DialogTitle>
          </DialogHeader>
          <div>
            <Label>{t("auth.password")}</Label>
            <Input type="password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setResetPwDialog(null)}>{t("common.cancel")}</Button>
            <Button onClick={handleResetPassword}>{t("common.confirm")}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
