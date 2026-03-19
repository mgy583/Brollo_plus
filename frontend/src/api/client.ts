import axios, { AxiosError } from "axios";
import { message } from "antd";
import { useAuthStore } from "../store/authStore";

export type ApiSuccess<T> = {
  success: true;
  data: T;
  message: string;
  timestamp: string;
  request_id: string;
};

export type ApiError = {
  success: false;
  error: { code: string; message: string; details?: unknown };
  timestamp: string;
  request_id: string;
};

export const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || "/api/v1",
  timeout: 30000,
  headers: { "Content-Type": "application/json", Accept: "application/json" }
});

api.interceptors.request.use((config) => {
  const token = useAuthStore.getState().accessToken;
  if (token) config.headers.Authorization = `Bearer ${token}`;
  config.headers["X-Request-ID"] = crypto.randomUUID();
  config.headers["X-Client-Version"] = "0.1.0";
  return config;
});

api.interceptors.response.use(
  (r) => r.data,
  async (error: AxiosError<ApiError>) => {
    const status = error.response?.status;

    if (status === 401) {
      const refreshed = await tryRefresh();
      if (refreshed && error.config) {
        return api.request(error.config);
      }
      useAuthStore.getState().clearSession();
      window.location.href = "/auth/login";
      return Promise.reject(error);
    }

    const msg = error.response?.data?.error?.message;
    if (msg) message.error(msg);
    else message.error("网络错误，请稍后再试");
    return Promise.reject(error);
  }
);

async function tryRefresh(): Promise<boolean> {
  const refreshToken = useAuthStore.getState().refreshToken;
  if (!refreshToken) return false;

  try {
    const r = await axios.post<ApiSuccess<{ access_token: string; expires_in: number }>>(
      "/api/v1/auth/refresh",
      { refresh_token: refreshToken },
      { baseURL: "" }
    );
    useAuthStore.getState().setSession({
      accessToken: r.data.data.access_token,
      refreshToken,
      user: useAuthStore.getState().user!
    });
    return true;
  } catch {
    return false;
  }
}

