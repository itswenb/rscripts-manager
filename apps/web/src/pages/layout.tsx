import { Outlet, Link, useNavigate } from "@tanstack/react-router";
import { clearCredentials } from "@/lib/auth";
import { Button } from "@/components/ui/button";

export function AuthenticatedLayout() {
  const navigate = useNavigate();

  function handleLogout() {
    clearCredentials();
    navigate({ to: "/login" });
  }

  return (
    <div className="min-h-screen flex">
      <aside className="w-56 border-r bg-white p-4 flex flex-col">
        <h2 className="text-lg font-bold mb-6">RFlow</h2>
        <nav className="flex flex-col gap-1 flex-1">
          <Link to="/projects" className="px-3 py-2 rounded hover:bg-gray-100 text-sm">
            Projects
          </Link>
          <Link to="/workflow-steps" className="px-3 py-2 rounded hover:bg-gray-100 text-sm">
            Workflow Steps
          </Link>
        </nav>
        <Button variant="ghost" size="sm" onClick={handleLogout}>
          Logout
        </Button>
      </aside>
      <main className="flex-1 p-6 bg-gray-50 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
