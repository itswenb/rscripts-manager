const CRED_KEY = "rflow_cred";

export function getCredentials(): string | null {
  return localStorage.getItem(CRED_KEY);
}

export function setCredentials(username: string, password: string): void {
  localStorage.setItem(CRED_KEY, btoa(`${username}:${password}`));
}

export function clearCredentials(): void {
  localStorage.removeItem(CRED_KEY);
}

export function isAuthenticated(): boolean {
  return getCredentials() !== null;
}
