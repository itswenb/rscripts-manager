import { createRootRoute, createRoute, createRouter, redirect, Outlet } from "@tanstack/react-router";
import { isAuthenticated } from "@/lib/auth";
import { LoginPage } from "@/pages/login";
import { AuthenticatedLayout } from "@/pages/layout";
import { ProjectsPage } from "@/pages/projects";
import { ProjectDashboardPage } from "@/pages/project-dashboard";
import { ProjectExplorerPage } from "@/pages/project-explorer";
import { ProjectRunsPage } from "@/pages/project-runs";
import { RunDetailPage } from "@/pages/run-detail";
import { PipelinesPage } from "@/pages/pipelines";
import { FileManagerPage } from "@/pages/file-manager";
import { UsersPage } from "@/pages/users";
import { AuditLogPage } from "@/pages/audit-log";

const rootRoute = createRootRoute({
  component: Outlet,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/login",
  component: LoginPage,
});

const authenticatedRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "authenticated",
  beforeLoad: () => {
    if (!isAuthenticated()) {
      throw redirect({ to: "/login" });
    }
  },
  component: AuthenticatedLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/projects" });
  },
});

const projectsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects",
  component: ProjectsPage,
});

const projectDetailRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId",
  beforeLoad: ({ params }) => {
    throw redirect({ to: "/projects/$projectId/dashboard", params });
  },
});

const projectDashboardRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/dashboard",
  component: ProjectDashboardPage,
});

const projectExplorerRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/explorer",
  component: ProjectExplorerPage,
});

const projectRunsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/runs",
  component: ProjectRunsPage,
});

const runDetailRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId/runs/$runId",
  component: RunDetailPage,
});

const workflowStepsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/workflow-steps",
  beforeLoad: () => {
    throw redirect({ to: "/pipelines" });
  },
});

const pipelinesRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/pipelines",
  component: PipelinesPage,
});

const filesRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/files",
  component: FileManagerPage,
});

const usersRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/users",
  component: UsersPage,
});

const auditLogRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/audit-log",
  component: AuditLogPage,
});

const routeTree = rootRoute.addChildren([
  loginRoute,
  authenticatedRoute.addChildren([
    indexRoute,
    projectsRoute,
    projectDetailRoute,
    projectDashboardRoute,
    projectExplorerRoute,
    projectRunsRoute,
    runDetailRoute,
    workflowStepsRoute,
    pipelinesRoute,
    filesRoute,
    usersRoute,
    auditLogRoute,
  ]),
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
