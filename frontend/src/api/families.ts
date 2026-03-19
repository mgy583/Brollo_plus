import { api, ApiSuccess } from "./client";

export type Family = {
  id: number;
  uuid: string;
  name: string;
  invite_code: string;
  owner_id: number;
  default_currency: string;
  description?: string | null;
  avatar?: string | null;
  member_count: number;
  created_at: string;
};

export type FamilyMember = {
  id: number;
  family_id: number;
  user_id: number;
  user_uuid: string;
  username: string;
  email: string;
  full_name?: string | null;
  role: "owner" | "admin" | "member" | "readonly";
  nickname?: string | null;
  joined_at: string;
};

export const familiesApi = {
  mine: () => api.get<unknown, ApiSuccess<{ family: Family | null }>>("/families/mine"),

  create: (payload: { name: string; default_currency?: string; description?: string; nickname?: string }) =>
    api.post<unknown, ApiSuccess<{ family: Family }>>("/families", payload),

  join: (payload: { invite_code: string; nickname?: string }) =>
    api.post<unknown, ApiSuccess<{ family: Family }>>("/families/join", payload),

  get: (id: number) => api.get<unknown, ApiSuccess<{ family: Family }>>(`/families/${id}`),

  update: (id: number, payload: { name?: string; default_currency?: string; description?: string; avatar?: string }) =>
    api.patch<unknown, ApiSuccess<{ family: Family }>>(`/families/${id}`, payload),

  members: (id: number) =>
    api.get<unknown, ApiSuccess<{ members: FamilyMember[] }>>(`/families/${id}/members`),

  updateMember: (fid: number, uid: number, payload: { role?: string; nickname?: string }) =>
    api.patch<unknown, ApiSuccess<{}>>(`/families/${fid}/members/${uid}`, payload),

  removeMember: (fid: number, uid: number) =>
    api.delete<unknown, ApiSuccess<{}>>(`/families/${fid}/members/${uid}`),

  refreshInvite: (id: number) =>
    api.post<unknown, ApiSuccess<{ invite_code: string }>>(`/families/${id}/invite`),

  leave: (id: number) =>
    api.post<unknown, ApiSuccess<{}>>(`/families/${id}/leave`),

  dissolve: (id: number) =>
    api.delete<unknown, ApiSuccess<{}>>(`/families/${id}`),
};
