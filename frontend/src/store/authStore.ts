import { create } from "zustand";
import { persist } from "zustand/middleware";

export type AuthUser = {
  id: string;
  username: string;
  email: string;
  full_name?: string | null;
  settings?: Record<string, unknown>;
};

type AuthState = {
  accessToken: string | null;
  refreshToken: string | null;
  user: AuthUser | null;
  setSession: (s: { accessToken: string; refreshToken: string; user: AuthUser }) => void;
  clearSession: () => void;
};

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      accessToken: null,
      refreshToken: null,
      user: null,
      setSession: ({ accessToken, refreshToken, user }) =>
        set({ accessToken, refreshToken, user }),
      clearSession: () => set({ accessToken: null, refreshToken: null, user: null }),
    }),
    { name: "brollo-auth" }
  )
);
