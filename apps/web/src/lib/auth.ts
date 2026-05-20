import { useAppStore } from "@/store";

export function getCredentials(): string | null {
  return useAppStore.getState().credentials;
}

export function isAuthenticated(): boolean {
  return getCredentials() !== null;
}
