import { api, ApiSuccess } from "./client";
import axios from "axios";

export type User = {
  id: string;
  username: string;
  email: string;
  full_name?: string | null;
  settings?: {
    default_currency?: string;
    timezone?: string;
    language?: string;
    theme?: string;
  };
  created_at?: string;
};

export const authApi = {
  register: (payload: {
    username: string;
    email: string;
    password: string;
    full_name?: string;
  }) =>
    api.post<unknown, ApiSuccess<{ user: User; tokens: { access_token: string; refresh_token: string; expires_in: number } }>>(
      "/auth/register",
      payload
    ),

  login: (payload: {
    username: string;
    password: string;
    device_info?: { device_type: string; device_name: string };
  }) =>
    api.post<unknown, ApiSuccess<{ user: User; tokens: { access_token: string; refresh_token: string; expires_in: number } }>>(
      "/auth/login",
      payload
    ),

  logout: (refresh_token: string) =>
    api.post<unknown, ApiSuccess<null>>("/auth/logout", { refresh_token }),

  refresh: (refresh_token: string) =>
    axios.post<ApiSuccess<{ access_token: string; expires_in: number }>>(
      "/api/v1/auth/refresh",
      { refresh_token }
    ),

  getMe: () =>
    api.get<unknown, ApiSuccess<User>>("/users/me"),

  updateMe: (payload: { full_name?: string; phone?: string }) =>
    api.patch<unknown, ApiSuccess<User>>("/users/me", payload),

  updateSettings: (payload: {
    default_currency?: string;
    timezone?: string;
    language?: string;
    theme?: string;
  }) =>
    api.put<unknown, ApiSuccess<User>>("/users/me/settings", payload),

  changePassword: (payload: { old_password: string; new_password: string }) =>
    api.post<unknown, ApiSuccess<null>>("/users/me/password", payload),
};
