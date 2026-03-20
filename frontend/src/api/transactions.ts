import { api, ApiSuccess } from "./client";

export type Transaction = {
  id: string;
  type: "expense" | "income" | "transfer";
  amount: number;
  currency: string;
  account_id: string;
  to_account_id?: string | null;
  category_id: string;
  description?: string | null;
  payee?: string | null;
  transaction_date: string;
  tags: string[];
  status: string;
  family_id?: string | null;
  recorder_id?: string | null;
  created_at?: string;
};

export const transactionsApi = {
  list: (params?: Record<string, unknown>) =>
    api.get<unknown, ApiSuccess<{
      transactions: Transaction[];
      pagination: { total: number; page: number; page_size: number; total_pages: number; has_next: boolean; has_prev: boolean };
    }>>("/transactions", { params }),

  listFamily: (familyId: number | string, params?: Record<string, unknown>) =>
    api.get<unknown, ApiSuccess<{
      transactions: Transaction[];
      pagination: { total: number; page: number; page_size: number; total_pages: number; has_next: boolean; has_prev: boolean };
    }>>(`/transactions/family/${familyId}`, { params }),

  create: (payload: {
    type: "expense" | "income" | "transfer";
    amount: number;
    currency: string;
    account_id: string;
    to_account_id?: string;
    category_id: string;
    description?: string;
    payee?: string;
    transaction_date: string;
    tags?: string[];
    notes?: string;
    family_id?: string;
  }) => api.post<unknown, ApiSuccess<{ id: string }>>("/transactions", payload),

  get: (id: string) =>
    api.get<unknown, ApiSuccess<Transaction>>(`/transactions/${id}`),

  update: (id: string, payload: Partial<Transaction>) =>
    api.patch<unknown, ApiSuccess< Record<string, never> >>(`/transactions/${id}`, payload),

  delete: (id: string) =>
    api.delete<unknown, ApiSuccess< Record<string, never> >>(`/transactions/${id}`),
};
