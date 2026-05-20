import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import i18n from "@/i18n";

type Theme = "light" | "dark" | "system";
type Locale = "zh" | "en";

interface AppState {
  theme: Theme;
  locale: Locale;
  credentials: string | null;
  setTheme: (theme: Theme) => void;
  setLocale: (locale: Locale) => void;
  login: (username: string, password: string) => void;
  logout: () => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      theme: "system",
      locale: "zh",
      credentials: null,
      setTheme: (theme) => set({ theme }),
      setLocale: (locale) => {
        i18n.changeLanguage(locale);
        set({ locale });
      },
      login: (username, password) =>
        set({ credentials: btoa(`${username}:${password}`) }),
      logout: () => set({ credentials: null }),
    }),
    {
      name: "rflow-store",
      storage: createJSONStorage(() => sessionStorage),
    }
  )
);
