# Rflow UI Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign the Rflow web frontend to match the Stitch design mockups — card-based projects page, rich project dashboard, enhanced file explorer with directory tree, detailed run execution view, and expanded sidebar navigation.

**Architecture:** Refactor existing pages in-place. Add new route for run detail. Expand sidebar nav to include Dashboard/Explorer/Workflows/Runs. Use recharts for the dashboard chart. Keep all data fetching via TanStack Query hooks.

**Tech Stack:** React 19, TanStack Router, TanStack Query, Tailwind CSS 3, shadcn/ui, lucide-react, recharts, Farm bundler

---

## Task 0: Install Dependencies

**Files:**
- Modify: `apps/web/package.json`

**Step 1: Install recharts for dashboard charts**

Run: `cd apps/web && pnpm add recharts`

**Step 2: Verify build still works**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add apps/web/package.json apps/web/pnpm-lock.yaml
git commit -m "feat(web): add recharts dependency for dashboard charts"
```

---

## Task 1: Add New shadcn/ui Components (Progress, RadioGroup, Checkbox)

**Files:**
- Create: `apps/web/src/components/ui/progress.tsx`
- Create: `apps/web/src/components/ui/radio-group.tsx`
- Create: `apps/web/src/components/ui/checkbox.tsx`

**Step 1: Create Progress component**

```tsx
// apps/web/src/components/ui/progress.tsx
import * as React from "react";
import { cn } from "@/lib/utils";

const Progress = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { value?: number; max?: number }
>(({ className, value = 0, max = 100, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("relative h-2 w-full overflow-hidden rounded-full bg-secondary", className)}
    {...props}
  >
    <div
      className="h-full bg-primary transition-all"
      style={{ width: `${(value / max) * 100}%` }}
    />
  </div>
));
Progress.displayName = "Progress";

export { Progress };
```

**Step 2: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add apps/web/src/components/ui/progress.tsx
git commit -m "feat(web): add Progress UI component"
```

---

## Task 2: Expand Sidebar Navigation

**Model hint:** `auto`

**Files:**
- Modify: `apps/web/src/pages/layout.tsx`

**Step 1: Update sidebar nav items to match Stitch design**

Replace the `navItems` array and update the layout to include the expanded navigation with Dashboard, Explorer, Workflows, Runs sections plus Documentation/Support secondary links:

```tsx
// apps/web/src/pages/layout.tsx
import { Outlet, Link, useNavigate, useRouterState } from "@tanstack/react-router";
import { clearCredentials } from "@/lib/auth";
import { LayoutGrid, FolderOpen, GitBranch, Play, LogOut, FileText, HelpCircle } from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/projects/$projectId/dashboard", label: "Dashboard", icon: LayoutGrid, matchPrefix: "/projects/" },
  { to: "/projects/$projectId/explorer", label: "Explorer", icon: FolderOpen, matchPrefix: "/projects/" },
  { to: "/workflow-steps", label: "Workflows", icon: GitBranch },
  { to: "/projects/$projectId/runs", label: "Runs", icon: Play, matchPrefix: "/projects/" },
];

const secondaryItems = [
  { label: "Documentation", icon: FileText },
  { label: "Support", icon: HelpCircle },
];
```

Note: The sidebar should show project-scoped nav when inside a project, and the projects list when at top level. The implementation should conditionally render based on whether a `projectId` param is present.

**Step 2: Update layout structure**

Add a top header bar with project name/version badge, search shortcut (⌘K), and user avatar. Keep the sidebar but widen slightly to accommodate the new items.

```tsx
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
      <aside className="w-56 border-r bg-card flex flex-col shrink-0">
        <div className="px-4 py-4 border-b">
          <span className="text-sm font-semibold tracking-tight">Rflow</span>
        </div>
        <nav className="flex-1 px-2 py-3 space-y-0.5">
          {/* Primary nav */}
          <div className="space-y-0.5">
            <NavLink to="/projects" icon={LayoutGrid} label="Projects" pathname={pathname} />
            <NavLink to="/workflow-steps" icon={GitBranch} label="Workflows" pathname={pathname} />
          </div>
          {/* Secondary nav */}
          <div className="pt-4 mt-4 border-t space-y-0.5">
            <span className="px-3 text-xs text-muted-foreground font-medium uppercase tracking-wider">Resources</span>
            {secondaryItems.map(({ label, icon: Icon }) => (
              <span key={label} className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground cursor-default">
                <Icon size={15} />
                {label}
              </span>
            ))}
          </div>
        </nav>
        <div className="px-2 py-3 border-t">
          <button onClick={handleLogout} className="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground w-full transition-colors">
            <LogOut size={15} />
            Logout
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
```

**Step 3: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add apps/web/src/pages/layout.tsx
git commit -m "feat(web): expand sidebar navigation with Rflow design"
```

---

## Task 3: Redesign Projects Page (Card Grid)

**Model hint:** `gemini`

**Files:**
- Modify: `apps/web/src/pages/projects.tsx`

**Step 1: Redesign projects page with card grid layout**

Replace the table-based layout with a card grid. Each card shows:
- Status badge (Success/Running/Failed/Idle) with semantic colors
- Project ID (PRJ-###)
- Title
- Description
- File count
- Last updated timestamp

Add filter controls (status dropdown), sort options, and grid/list view toggle.

Key implementation details:
- Status is derived from the most recent run's status for that project (requires a new query or backend field — for now, use "Idle" as default since the API doesn't return status)
- Card grid uses `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4`
- Keep the existing create project dialog
- Add "Add New Project" button prominently

```tsx
// Status badge color mapping
const STATUS_CONFIG = {
  success: { label: "Success", class: "bg-green-50 text-green-700 border-green-200" },
  running: { label: "Running", class: "bg-blue-50 text-blue-700 border-blue-200" },
  failed: { label: "Failed", class: "bg-red-50 text-red-700 border-red-200" },
  idle: { label: "Idle", class: "bg-gray-50 text-gray-500 border-gray-200" },
};
```

**Step 2: Verify build and test in browser**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds. Projects page shows card grid.

**Step 3: Commit**

```bash
git add apps/web/src/pages/projects.tsx
git commit -m "feat(web): redesign projects page with card grid layout"
```

---

## Task 4: Add Project Dashboard Route and Page

**Model hint:** `auto`

**Files:**
- Create: `apps/web/src/pages/project-dashboard.tsx`
- Modify: `apps/web/src/router.tsx`
- Modify: `apps/web/src/pages/project-detail.tsx`

**Step 1: Create project dashboard page**

The dashboard is the new default view when entering a project. It shows:
1. **Header**: Project name with version badge, "Add New Run" button
2. **Overview cards** (4 cards in a row): System status, Active Workflows count, Failed Runs (24h), Compute Hours
3. **Quick Actions**: Start New Run, Upload Datasets buttons
4. **Chart**: Line chart showing ScriptRuns over last 7 days (use recharts `LineChart`)
5. **Storage utilization**: Progress bar showing used/total
6. **Recent executions table**: Last 4-5 runs with status, duration, started time
7. **Activity feed**: Chronological log of recent events

```tsx
// apps/web/src/pages/project-dashboard.tsx
import { useParams } from "@tanstack/react-router";
import { useProject } from "@/lib/queries/projects";
import { useRuns } from "@/lib/queries/runs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { Progress } from "@/components/ui/progress";
import { Play, Upload, Activity, HardDrive, AlertTriangle, CheckCircle } from "lucide-react";
```

For the chart data, aggregate runs by day from the runs query. For overview cards, compute from runs data:
- Active Workflows: count of distinct workflow_step_ids with running status
- Failed Runs (24h): count of failed runs in last 24h
- Compute Hours: sum of durations

**Step 2: Add route for dashboard**

In `router.tsx`, add:
```tsx
const projectDashboardRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/dashboard",
  component: ProjectDashboardPage,
});
```

Update `project-detail.tsx` to redirect `/projects/$projectId` to `/projects/$projectId/dashboard`.

**Step 3: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add apps/web/src/pages/project-dashboard.tsx apps/web/src/router.tsx apps/web/src/pages/project-detail.tsx
git commit -m "feat(web): add project dashboard page with charts and overview cards"
```

---

## Task 5: Redesign File Explorer with Directory Tree

**Model hint:** `gemini`

**Files:**
- Modify: `apps/web/src/components/file-manager.tsx`
- Create: `apps/web/src/pages/project-explorer.tsx`
- Modify: `apps/web/src/router.tsx`

**Step 1: Create dedicated explorer page route**

Add `/projects/$projectId/explorer` route that renders the enhanced file explorer.

**Step 2: Redesign file explorer layout**

Split into two panels:
- **Left panel (w-56)**: Directory tree showing folder hierarchy (uploads/, workspace/, runs/, trash/)
- **Right panel (flex-1)**: File table with columns: Name (with type icon), Type, Size, Modified, Actions

Add:
- Breadcrumb navigation at top
- Toolbar with Upload and "New File" buttons
- Status bar at bottom showing selection count and total size
- File type icons (Python, CSV, JSON, Image) based on extension

```tsx
// Directory tree component
function DirectoryTree({ projectId, onSelect, selectedId }: {
  projectId: string;
  onSelect: (id: string | null, name: string) => void;
  selectedId: string | null;
}) {
  const { data: rootFiles } = useFiles(projectId, null);
  const directories = rootFiles?.filter(f => f.is_directory) ?? [];
  // Render recursive tree...
}
```

**Step 3: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add apps/web/src/components/file-manager.tsx apps/web/src/pages/project-explorer.tsx apps/web/src/router.tsx
git commit -m "feat(web): redesign file explorer with directory tree sidebar"
```

---

## Task 6: Add Run Detail Page

**Model hint:** `auto`

**Files:**
- Create: `apps/web/src/pages/run-detail.tsx`
- Modify: `apps/web/src/router.tsx`
- Modify: `apps/web/src/lib/queries/runs.ts`

**Step 1: Add useRun single-run query hook**

```tsx
// In runs.ts, add:
export function useRun(projectId: string, runId: string) {
  return useQuery({
    queryKey: ["runs", projectId, runId],
    queryFn: () => api.get<ScriptRun>(`/projects/${projectId}/runs/${runId}`),
    refetchInterval: (query) => {
      const data = query.state.data;
      return data?.status === "running" || data?.status === "pending" ? 2000 : false;
    },
  });
}
```

**Step 2: Create run detail page**

Layout:
1. **Breadcrumb**: WorkflowStep name > Run ID
2. **Action buttons**: Abort (if running), more options menu
3. **Metadata panel**: Status badge, Start Time, Duration (live updating if running), Compute Node (placeholder)
4. **Progress section**: "Step X of Y" with progress bar (parse from stdout if available, otherwise show indeterminate)
5. **Tabbed content**:
   - **Live Logs tab**: Monospace log viewer with timestamped entries, severity coloring ([INFO] blue, [WARN] yellow, [ERROR] red), auto-scroll toggle, download button, fullscreen button
   - **Results tab**: Output files list with download links
   - **Configuration tab**: Display inputs and params as JSON

```tsx
// Log line parsing
function parseLogLine(line: string) {
  const match = line.match(/^\[(\d{2}:\d{2}:\d{2})\]\s*\[(\w+)\]\s*(.*)/);
  if (!match) return { time: "", level: "INFO", message: line };
  return { time: match[1], level: match[2], message: match[3] };
}

const LOG_COLORS = {
  INFO: "text-blue-600",
  OK: "text-green-600",
  WARN: "text-yellow-600",
  ERROR: "text-red-600",
};
```

**Step 3: Add route**

```tsx
const runDetailRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/runs/$runId",
  component: RunDetailPage,
});
```

**Step 4: Update runs panel to link to detail page**

In `runs-panel.tsx`, change the Eye button to navigate to `/projects/$projectId/runs/$runId` instead of opening a dialog.

**Step 5: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add apps/web/src/pages/run-detail.tsx apps/web/src/router.tsx apps/web/src/lib/queries/runs.ts apps/web/src/components/runs-panel.tsx
git commit -m "feat(web): add dedicated run detail page with log viewer"
```

---

## Task 7: Add Runs List Page

**Model hint:** `auto`

**Files:**
- Create: `apps/web/src/pages/project-runs.tsx`
- Modify: `apps/web/src/router.tsx`

**Step 1: Create runs list page**

A dedicated page at `/projects/$projectId/runs` that shows:
- Header with "Runs" title and "New Run" button
- Filter controls: status filter, date range
- Table of all runs with columns: Run ID, Script, Status, Duration, Started
- Each row links to the run detail page
- Status badges with semantic colors

This replaces the tab-based approach in project-detail.

**Step 2: Add route**

```tsx
const projectRunsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/runs",
  component: ProjectRunsPage,
});
```

**Step 3: Verify build**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add apps/web/src/pages/project-runs.tsx apps/web/src/router.tsx
git commit -m "feat(web): add dedicated runs list page"
```

---

## Task 8: Update Project Detail as Router Redirect

**Files:**
- Modify: `apps/web/src/pages/project-detail.tsx`
- Modify: `apps/web/src/router.tsx`

**Step 1: Make project detail redirect to dashboard**

The `/projects/$projectId` route should now redirect to `/projects/$projectId/dashboard`. The old tabs-based detail page is replaced by the new sub-routes (dashboard, explorer, runs).

```tsx
// In router.tsx, update projectDetailRoute:
const projectDetailRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId",
  beforeLoad: ({ params }) => {
    throw redirect({ to: "/projects/$projectId/dashboard", params });
  },
});
```

**Step 2: Update sidebar to show project-scoped navigation**

When inside a project (URL matches `/projects/$projectId/*`), the sidebar should show:
- Dashboard, Explorer, Workflows, Runs links scoped to that project
- Back to Projects link at top

**Step 3: Verify build and all routes work**

Run: `cd apps/web && pnpm build`
Expected: Build succeeds. Navigation between project sub-pages works.

**Step 4: Commit**

```bash
git add apps/web/src/pages/project-detail.tsx apps/web/src/router.tsx apps/web/src/pages/layout.tsx
git commit -m "feat(web): restructure project routes with sub-navigation"
```

---

## Task 9: Polish and Visual Refinements

**Model hint:** `gemini`

**Files:**
- Modify: `apps/web/src/index.css`
- Modify: various component files

**Step 1: Add Inter font import**

Add Inter font via CSS import for the clean sans-serif look matching the Stitch design.

**Step 2: Add monospace font for logs**

Use `font-mono` (JetBrains Mono or system monospace) for log displays and code.

**Step 3: Refine color palette**

Ensure the neutral gray/slate palette with indigo accent matches the Stitch design. Update CSS variables if needed.

**Step 4: Verify build and visual check**

Run: `cd apps/web && pnpm dev`
Expected: All pages render correctly with consistent styling.

**Step 5: Commit**

```bash
git add apps/web/src/index.css
git commit -m "feat(web): polish typography and color palette"
```

---

## Summary of New Routes

| Path | Page | Description |
|------|------|-------------|
| `/projects` | ProjectsPage | Card grid of all projects |
| `/projects/$projectId` | Redirect | → dashboard |
| `/projects/$projectId/dashboard` | ProjectDashboardPage | Overview, charts, recent runs |
| `/projects/$projectId/explorer` | ProjectExplorerPage | File tree + table |
| `/projects/$projectId/runs` | ProjectRunsPage | Runs list |
| `/projects/$projectId/runs/$runId` | RunDetailPage | Execution detail + logs |
| `/workflow-steps` | WorkflowStepsPage | (existing) |

## New Dependencies

- `recharts` — lightweight React charting library for the dashboard line chart and storage visualization

## shadcn/ui Components to Add

- `Progress` — for run progress bars and storage utilization
- (RadioGroup and Checkbox if workflow configuration page is added later)
