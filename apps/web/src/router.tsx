import { createRootRoute, createRoute, createRouter, redirect, Outlet } from "@tanstack/react-router";
import { isAuthenticated } from "@/lib/auth";
import { LoginPage } from "@/pages/login";
import { AuthenticatedLayout } from "@/pages/layout";

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

export const projectsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects",
});

export const projectDetailRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/projects/$projectId",
});

export const workflowStepsRoute = createRoute({
  getParentRoute: () => authenticatedRoute,
  path: "/workflow-steps",
});

const routeTree = rootRoute.addChildren([
  loginRoute,
  authenticatedRoute.addChildren([
    indexRoute,
    projectsRoute,
    projectDetailRoute,
    workflowStepsRoute,
  ]),
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
