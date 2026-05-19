import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { setCredentials } from "@/lib/auth";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const cred = btoa(`${username}:${password}`);
      const res = await fetch("/api/projects", {
        headers: { Authorization: `Basic ${cred}` },
      });
      if (res.status === 401) {
        setError("Invalid credentials");
        return;
      }
      setCredentials(username, password);
      navigate({ to: "/" });
    } catch {
      setError("Connection failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <form onSubmit={handleSubmit} className="w-full max-w-sm space-y-4 p-6 bg-white rounded-lg shadow">
        <h1 className="text-2xl font-bold text-center">RFlow</h1>
        {error && <p className="text-sm text-red-600">{error}</p>}
        <Input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          autoFocus
        />
        <Input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <Button type="submit" className="w-full" disabled={loading}>
          {loading ? "Logging in..." : "Login"}
        </Button>
      </form>
    </div>
  );
}
