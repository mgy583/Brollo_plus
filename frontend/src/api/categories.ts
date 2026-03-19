import { api, ApiSuccess } from "./client";

export type Category = {
  id: string;
  name: string;
  type: "expense" | "income";
  icon?: string | null;
  color?: string | null;
  parent_id?: string | null;
  order: number;
  is_system: boolean;
  is_archived: boolean;
  children: Category[];
};

export const categoriesApi = {
  list: (params?: { type?: string }) =>
    api.get<unknown, ApiSuccess<{ categories: Category[] }>>("/categories", { params }),
  create: (payload: {
    name: string;
    type: string;
    icon?: string;
    color?: string;
    parent_id?: string | null;
  }) => api.post<unknown, ApiSuccess<{ id: string }>>("/categories", payload),
  update: (id: string, payload: { name?: string; icon?: string; color?: string }) =>
    api.patch<unknown, ApiSuccess<{}>>(`/categories/${id}`, payload),
  remove: (id: string) => api.delete<unknown, ApiSuccess<{}>>(`/categories/${id}`),
};
