import { Outlet, Link, useNavigate, useRouterState } from "@tanstack/react-router";
import { clearCredentials } from "@/lib/auth";
import { LayoutGrid, GitBranch, LogOut } from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/projects", label: "Projects", icon: LayoutGrid },
  { to: "/workflow-steps", label: "Workflow Steps", icon: GitBranch },
];

export function AuthenticatedLayout() {
  const navigate = useNavigate();
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  function handleLogout() {
    clearCredentials();
    navigate({ to: "/login" });
  }

  return (
    <div className="min-h-screen flex bg-background">
      <aside className="w-52 border-r bg-card flex flex-col shrink-0">
        <div className="px-4 py-4 border-b">
          <span className="text-sm font-semibold tracking-tight">RFlow</span>
        </div>
        <nav className="flex-1 px-2 py-3 space-y-0.5">
          {navItems.map(({ to, label, icon: Icon }) => {
            const active = pathname === to || pathname.startsWith(to + "/");
            return (
              <Link
                key={to}
                to={to}
                className={cn(
                  "flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-colors",
                  active
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                )}
              >
                <Icon size={15} />
                {label}
              </Link>
            );
          })}
        </nav>
        <div className="px-2 py-3 border-t">
          <button
            onClick={handleLogout}
            className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground w-full transition-colors"
          >
            <LogOut size={15} />
            Logout
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        <div className="p-6 max-w-5xl">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
