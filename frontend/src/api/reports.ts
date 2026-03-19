import { api, ApiSuccess } from "./client";

export const reportsApi = {
  overview: (params: { start_date: string; end_date: string }) =>
    api.get<unknown, ApiSuccess<{
      total_income: number;
      total_expense: number;
      net: number;
      transaction_count: number;
      top_expense_categories: { category_id: string; amount: number; count: number }[];
    }>>("/reports/overview", { params }),

  categories: (params: { type: string; start_date: string; end_date: string }) =>
    api.get<unknown, ApiSuccess<{
      categories: { category_id: string; amount: number; count: number; percentage: number }[];
      total: number;
    }>>("/reports/categories", { params }),

  trend: (params: { start_date: string; end_date: string; interval?: string }) =>
    api.get<unknown, ApiSuccess<{
      series: { date: string; income: number; expense: number; net: number }[];
    }>>("/reports/trend", { params }),

  accounts: (params: { start_date: string; end_date: string }) =>
    api.get<unknown, ApiSuccess<{
      accounts: { account_id: string; income: number; expense: number; net: number }[];
    }>>("/reports/accounts", { params }),
};
