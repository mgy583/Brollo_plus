import { api, ApiSuccess } from "./client";

export type Account = {
  id: string;
  name: string;
  type: string;
  currency: string;
  initial_balance: number;
  current_balance: number;
  icon?: string | null;
  color?: string | null;
  description?: string | null;
  status: "active" | "archived";
  family_id?: string | null;
  visibility?: "private" | "family" | null;
  created_at?: string;
  updated_at?: string;
};

export const accountsApi = {
  list: (params?: { status?: string; type?: string; currency?: string }) =>
    api.get<unknown, ApiSuccess<{
      accounts: Account[];
      summary: { total_assets: number; total_liabilities: number; net_worth: number };
    }>>("/accounts", { params }),

  listFamily: (familyId: number | string) =>
    api.get<unknown, ApiSuccess<{
      accounts: Account[];
      summary: { total_assets: number; total_liabilities: number; net_worth: number };
    }>>(`/accounts/family/${familyId}`),

  create: (payload: {
    name: string;
    type: string;
    currency: string;
    initial_balance: number;
    icon?: string;
    color?: string;
    description?: string;
    family_id?: string;
    visibility?: "private" | "family";
  }) => api.post<unknown, ApiSuccess<{ id: string }>>("/accounts", payload),

  update: (id: string, payload: {
    name?: string;
    icon?: string;
    color?: string;
    description?: string;
    status?: string;
    visibility?: string;
  }) => api.patch<unknown, ApiSuccess< Record<string, never> >>(`/accounts/${id}`, payload),

  remove: (id: string) => api.delete<unknown, ApiSuccess< Record<string, never> >>(`/accounts/${id}`),
};
