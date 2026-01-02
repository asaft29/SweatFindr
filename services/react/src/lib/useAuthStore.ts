import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AuthState, User, UserRole } from "../lib/types";
import { authService } from "./authService";
import { isTokenExpired, decodeJwt } from "./tokenUtils";

interface AuthStore extends AuthState {
  login: (username: string, password: string) => Promise<void>;
  register: (
    email: string,
    password: string,
    role: string
  ) => Promise<{ userId?: number }>;
  logout: () => Promise<void>;
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  clearAuth: () => void;
  checkTokenExpiration: () => void;
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,

      login: async (username: string, password: string) => {
        set({ isLoading: true });
        try {
          const response = await authService.login({ username, password });

          if (response.success && response.token_value) {
            const token = response.token_value;
            const decoded = decodeJwt(token);

            const user: User = {
              id: Number(decoded?.sub) || 0,
              email: username,
              role: (decoded?.role ?? "client") as UserRole,
              emailVerified: true,
            };

            set({
              token,
              user,
              isAuthenticated: true,
              isLoading: false,
            });

            localStorage.setItem("auth_token", token);
          } else {
            throw new Error(response.message || "Login failed");
          }
        } catch (error) {
          set({ isLoading: false });
          throw error;
        }
      },

      register: async (email: string, password: string, role: string) => {
        set({ isLoading: true });
        try {
          const response = await authService.register({
            email,
            password,
            role,
          });

          if (response.success) {
            if (response.token_value && response.user_id) {
              const user: User = {
                id: response.user_id,
                email,
                role: role as UserRole,
                emailVerified: false,
              };

              set({
                token: response.token_value,
                user,
                isAuthenticated: true,
                isLoading: false,
              });

              localStorage.setItem("auth_token", response.token_value);
            }

            return { userId: response.user_id };
          } else {
            throw new Error(response.message || "Registration failed");
          }
        } catch (error) {
          set({ isLoading: false });
          throw error;
        }
      },

      logout: async () => {
        const { token } = get();
        if (token) {
          try {
            await authService.logout(token);
          } catch (error) {
            console.error("Logout failed:", error);
          }
        }

        get().clearAuth();
      },

      setUser: (user) => set({ user, isAuthenticated: !!user }),

      setToken: (token) => {
        set({ token });
        if (token) {
          localStorage.setItem("auth_token", token);
        } else {
          localStorage.removeItem("auth_token");
        }
      },

      clearAuth: () => {
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          isLoading: false,
        });
        localStorage.removeItem("auth_token");
        localStorage.removeItem("user");
        import("./clientService").then(({ clientService }) => {
          clientService.clearCache();
        });
      },

      checkTokenExpiration: () => {
        const { token } = get();
        if (token && isTokenExpired(token)) {
          console.log("Token expired, logging out...");
          get().clearAuth();
        }
      },
    }),
    {
      name: "auth-storage",
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.checkTokenExpiration();
        }
      },
    }
  )
);
