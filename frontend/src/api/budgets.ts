import { api, ApiSuccess } from "./client";

export type Budget = {
  id: string;
  name: string;
  type: string;
  amount: number;
  currency: string;
  spent: number;
  remaining: number;
  progress: number;
  status: string;
  start_date: string;
  end_date: string;
  category_ids: string[];
  account_ids: string[];
  family_id?: string | null;
  scope?: "personal" | "family" | null;
  created_at?: string;
  updated_at?: string;
};

export const budgetsApi = {
  list: (params?: { status?: string; type?: string }) =>
    api.get<unknown, ApiSuccess<{ budgets: Budget[] }>>("/budgets", { params }),

  listFamily: (familyId: number | string, params?: { status?: string; type?: string }) =>
    api.get<unknown, ApiSuccess<{ budgets: Budget[] }>>(`/budgets/family/${familyId}`, { params }),

  create: (payload: {
    name: string;
    type: string;
    start_date: string;
    end_date: string;
    amount: number;
    currency?: string;
    category_ids?: string[];
    account_ids?: string[];
    family_id?: string;
    scope?: "personal" | "family";
  }) => api.post<unknown, ApiSuccess<{ id: string }>>("/budgets", payload),

  get: (id: string) =>
    api.get<unknown, ApiSuccess<Budget>>(`/budgets/${id}`),

  update: (id: string, payload: {
    name?: string;
    amount?: number;
    status?: string;
    end_date?: string;
    category_ids?: string[];
  }) => api.patch<unknown, ApiSuccess< Record<string, never> >>(`/budgets/${id}`, payload),

  remove: (id: string) =>
    api.delete<unknown, ApiSuccess< Record<string, never> >>(`/budgets/${id}`),
};
