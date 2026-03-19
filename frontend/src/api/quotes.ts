import { api, ApiSuccess } from "./client";

export const quotesApi = {
  getRates: (params?: { base?: string; targets?: string }) =>
    api.get<unknown, ApiSuccess<{ base: string; rates: Record<string, number>; updated_at: string }>>(
      "/quotes/exchange-rates",
      { params }
    ),
  getRateHistory: (params: { base: string; target: string; start_date?: string; end_date?: string }) =>
    api.get<unknown, ApiSuccess<{ base: string; target: string; history: { date: string; rate: number }[] }>>(
      "/quotes/exchange-rates/history",
      { params }
    ),
  getNetWorth: (params?: { currency?: string }) =>
    api.get<unknown, ApiSuccess<{ net_worth: number; currency: string; accounts: unknown[] }>>(
      "/quotes/net-worth",
      { params }
    ),
};
