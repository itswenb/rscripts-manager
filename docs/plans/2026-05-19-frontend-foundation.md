# Frontend Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the frontend application shell — TanStack Router, TanStack Query, auth context with JWT, and a sidebar layout. This is the foundation all feature pages build on.

**Architecture:** File-based route structure under `src/routes/`. Auth state stored in React context backed by localStorage token. TanStack Query for all API calls with a shared `api` client that injects the Bearer token. Sidebar layout with navigation to Projects, Workflow Steps.

**Tech Stack:** @tanstack/react-router, @tanstack/react-query, React 19, TailwindCSS, shadcn/ui, lucide-react

---

### Task 1: Install frontend dependencies

**Files:**

- Modify: `apps/web/package.json`

**Step 1: Install packages**

```bash
cd apps/web
pnpm add @tanstack/react-router @tanstack/react-query
```

**Step 2: Verify dev server starts**

Run: `cd apps/web && pnpm dev`
Expected: Compiles without errors, serves on port 4000

**Step 3: Commit**

```bash
git add apps/web/package.json apps/web/pnpm-lock.yaml
git commit -m "feat(web): add TanStack Router and Query dependencies"
```

---

### Task 2: Create API client with auth token injection

**Files:**

- Create: `apps/web/src/lib/api.ts`
- Create: `apps/web/src/lib/auth.ts`

**Step 1: Write auth token helpers**

`apps/web/src/lib/auth.ts`:

```typescript
const TOKEN_KEY = "rflow_token";

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

export function isAuthenticated(): boolean {
  return getToken() !== null;
}
```

**Step 2: Write API client**

`apps/web/src/lib/api.ts`:

```typescript
import { getToken, clearToken } from "./auth";

const BASE_URL = "/api";

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  if (!(options.body instanceof FormData)) {
    headers["Content-Type"] = "application/json";
  }

  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (res.status === 401) {
    clearToken();
    window.location.href = "/login";
    throw new Error("Unauthorized");
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.message || `Request failed: ${res.status}`);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body?: unknown) =>
    request<T>(path, {
      method: "POST",
      body: body instanceof FormData ? body : JSON.stringify(body),
    }),
  patch: <T>(path: string, body: unknown) =>
    request<T>(path, { method: "PATCH", body: JSON.stringify(body) }),
  delete: <T>(path: string) => request<T>(path, { method: "DELETE" }),
};
```

**Step 3: Verify compilation**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 4: Commit**

```bash
git add apps/web/src/lib/api.ts apps/web/src/lib/auth.ts
git commit -m "feat(web): add API client with JWT token injection"
```

---

### Task 3: Set up TanStack Router with route tree

**Files:**

- Create: `apps/web/src/routes/__root.tsx`
- Create: `apps/web/src/routes/login.tsx`
- Create: `apps/web/src/routes/_authenticated.tsx`
- Create: `apps/web/src/routes/_authenticated/index.tsx`
- Create: `apps/web/src/router.ts`
- Modify: `apps/web/src/main.tsx`

**Step 1: Create root route**

`apps/web/src/routes/__root.tsx`:

```tsx
import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: () => <Outlet />,
});
```

**Step 2: Create login route**

`apps/web/src/routes/login.tsx`:

```tsx
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { api } from "@/lib/api";
import { setToken } from "@/lib/auth";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/login")({
  component: LoginPage,
});

function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    try {
      const res = await api.post<{ token: string }>("/login", { username, password });
      setToken(res.token);
      navigate({ to: "/" });
    } catch (err: any) {
      setError(err.message || "Login failed");
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <form onSubmit={handleSubmit} className="w-full max-w-sm space-y-4 p-6 bg-white rounded-lg shadow">
        <h1 className="text-2xl font-bold text-center">RFlow</h1>
        {error && <p className="text-sm text-red-600">{error}</p>}
        <input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className="w-full px-3 py-2 border rounded-md"
          autoFocus
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          className="w-full px-3 py-2 border rounded-md"
        />
        <Button type="submit" className="w-full">Login</Button>
      </form>
    </div>
  );
}
```

**Step 3: Create authenticated layout route**

`apps/web/src/routes/_authenticated.tsx`:

```tsx
import { createFileRoute, Outlet, redirect, Link, useNavigate } from "@tanstack/react-router";
import { isAuthenticated, clearToken } from "@/lib/auth";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/_authenticated")({
  beforeLoad: () => {
    if (!isAuthenticated()) {
      throw redirect({ to: "/login" });
    }
  },
  component: AuthenticatedLayout,
});

function AuthenticatedLayout() {
  const navigate = useNavigate();

  function handleLogout() {
    clearToken();
    navigate({ to: "/login" });
  }

  return (
    <div className="min-h-screen flex">
      <aside className="w-56 border-r bg-white p-4 flex flex-col gap-2">
        <h2 className="text-lg font-bold mb-4">RFlow</h2>
        <Link to="/" className="px-3 py-2 rounded hover:bg-gray-100 text-sm">
          Projects
        </Link>
        <Link to="/workflow-steps" className="px-3 py-2 rounded hover:bg-gray-100 text-sm">
          Workflow Steps
        </Link>
        <div className="mt-auto">
          <Button variant="ghost" size="sm" onClick={handleLogout} className="w-full">
            Logout
          </Button>
        </div>
      </aside>
      <main className="flex-1 p-6 bg-gray-50">
        <Outlet />
      </main>
    </div>
  );
}
```

**Step 4: Create index (dashboard) route**

`apps/web/src/routes/_authenticated/index.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_authenticated/")({
  component: () => <div><h1 className="text-2xl font-bold">Projects</h1><p className="text-gray-500 mt-2">Coming soon</p></div>,
});
```

**Step 5: Create router configuration**

`apps/web/src/router.ts`:

```typescript
import { createRouter } from "@tanstack/react-router";
import { Route as rootRoute } from "./routes/__root";
import { Route as loginRoute } from "./routes/login";
import { Route as authenticatedRoute } from "./routes/_authenticated";
import { Route as indexRoute } from "./routes/_authenticated/index";

const routeTree = rootRoute.addChildren([
  loginRoute,
  authenticatedRoute.addChildren([indexRoute]),
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
```

**Step 6: Update main.tsx**

`apps/web/src/main.tsx`:

```tsx
import React from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { router } from "./router";
import "./index.css";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1, staleTime: 30_000 } },
});

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>
);
```

**Step 7: Remove old App.tsx (no longer used)**

Delete `apps/web/src/App.tsx`.

**Step 8: Verify**

Run: `cd apps/web && pnpm build`
Expected: PASS

**Step 9: Commit**

```bash
git add apps/web/src/ && git add -u apps/web/src/
git commit -m "feat(web): set up TanStack Router with login and authenticated layout"
```

---

### Task 4: Add shadcn/ui Input and Card components

**Files:**

- Create: `apps/web/src/components/ui/input.tsx`
- Create: `apps/web/src/components/ui/card.tsx`

**Step 1: Create Input component**

`apps/web/src/components/ui/input.tsx`:

```tsx
import * as React from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";

export { Input };
```

**Step 2: Create Card component**

`apps/web/src/components/ui/card.tsx`:

```tsx
import * as React from "react";
import { cn } from "@/lib/utils";

const Card = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("rounded-lg border bg-card text-card-foreground shadow-sm", className)} {...props} />
  )
);
Card.displayName = "Card";

const CardHeader = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("flex flex-col space-y-1.5 p-6", className)} {...props} />
  )
);
CardHeader.displayName = "CardHeader";

const CardTitle = React.forwardRef<HTMLParagraphElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h3 ref={ref} className={cn("text-lg font-semibold leading-none tracking-tight", className)} {...props} />
  )
);
CardTitle.displayName = "CardTitle";

const CardContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("p-6 pt-0", className)} {...props} />
  )
);
CardContent.displayName = "CardContent";

export { Card, CardHeader, CardTitle, CardContent };
```

**Step 3: Commit**

```bash
git add apps/web/src/components/ui/
git commit -m "feat(web): add Input and Card shadcn components"
```

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1 | Dependencies |
| 2 | 2 | API client + auth helpers |
| 3 | 3 | Router + layout + login page |
| 4 | 4 | UI components |

## Notes

- Farm dev server proxies `/api` to `localhost:4001` (already configured in `farm.config.ts`).
- TanStack Router uses file-based route convention with manual route tree (no code-gen plugin for Farm).
- Auth redirect: unauthenticated users hitting `/_authenticated/*` routes get redirected to `/login`.
- QueryClient configured with 30s stale time and 1 retry.
- This plan does NOT include feature pages (projects, files, etc.) — those are in separate plans.
