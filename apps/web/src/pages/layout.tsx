import { Outlet, Link, useNavigate, useRouterState } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { useAppStore } from "@/store";
import {
  LayoutGrid, GitBranch, LogOut, FileText, HelpCircle,
  FolderOpen, Users, ClipboardList, Sun, Moon, Monitor, Globe,
} from "lucide-react";
import { cn } from "@/lib/utils";

export function AuthenticatedLayout() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;
  const { theme, locale, setTheme, setLocale, logout } = useAppStore();

  const navItems = [
    { to: "/projects", label: t("nav.projects"), icon: LayoutGrid },
    { to: "/pipelines", label: t("nav.workflows"), icon: GitBranch },
    { to: "/files", label: t("nav.fileManager"), icon: FolderOpen },
    { to: "/users", label: t("nav.users"), icon: Users },
    { to: "/audit-log", label: t("nav.auditLog"), icon: ClipboardList },
  ];

  function handleLogout() {
    logout();
    navigate({ to: "/login" });
  }

  function cycleTheme() {
    const next = theme === "light" ? "dark" : theme === "dark" ? "system" : "light";
    setTheme(next);
  }

  function toggleLocale() {
    setLocale(locale === "zh" ? "en" : "zh");
  }

  const ThemeIcon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;

  return (
    <div className="min-h-screen flex bg-background">
      <aside className="w-56 border-r bg-card flex flex-col shrink-0">
        <div className="px-4 py-4 border-b">
          <span className="text-sm font-semibold tracking-tight">Rflow</span>
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
          <div className="pt-4 mt-4 border-t space-y-0.5">
            <span className="px-3 text-xs text-muted-foreground font-medium uppercase tracking-wider">
              {t("nav.resources")}
            </span>
            <span className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground cursor-default">
              <FileText size={15} />
              {t("nav.documentation")}
            </span>
            <span className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground cursor-default">
              <HelpCircle size={15} />
              {t("nav.support")}
            </span>
          </div>
        </nav>
        <div className="px-2 py-3 border-t space-y-1">
          <div className="flex items-center gap-1 px-1">
            <button
              onClick={cycleTheme}
              className="flex items-center gap-1.5 px-2 py-1.5 rounded text-xs text-muted-foreground hover:bg-accent"
              title={t(`theme.${theme}`)}
            >
              <ThemeIcon size={14} />
            </button>
            <button
              onClick={toggleLocale}
              className="flex items-center gap-1.5 px-2 py-1.5 rounded text-xs text-muted-foreground hover:bg-accent"
            >
              <Globe size={14} />
              <span>{locale === "zh" ? "中" : "EN"}</span>
            </button>
          </div>
          <button
            onClick={handleLogout}
            className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground w-full transition-colors"
          >
            <LogOut size={15} />
            {t("common.logout")}
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
