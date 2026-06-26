import { getAccessToken } from "../auth/session";

export type RoleDescriptor = {
  code: string;
  label_key: string;
};

export type AdminUser = {
  id: string;
  email: string;
  first_name: string | null;
  last_name: string | null;
  created_at: string;
  last_login_at: string | null;
  roles: string[];
};

export type AdminUsersResponse = {
  items: AdminUser[];
  total: number;
  offset: number;
  limit: number;
};

export type AdminUsersQuery = {
  offset: number;
  searchEmail: string;
  role: string;
  sort: "id" | "email" | "created_at" | "last_login_at";
  direction: "asc" | "desc";
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function authHeaders(contentType = false): HeadersInit {
  const token = getAccessToken();
  if (!token) {
    throw new Error("missing access token");
  }

  return {
    Authorization: `Bearer ${token}`,
    ...(contentType ? { "Content-Type": "application/json" } : {}),
  };
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function listRoles(): Promise<RoleDescriptor[]> {
  const payload = await fetchJson<{ roles: RoleDescriptor[] }>(`${apiBaseUrl()}/admin/roles`, {
    headers: authHeaders(),
  });
  return payload.roles;
}

export async function listUsers(query: AdminUsersQuery): Promise<AdminUsersResponse> {
  const params = new URLSearchParams({
    offset: String(query.offset),
    limit: "30",
    sort: query.sort,
    direction: query.direction,
  });
  if (query.searchEmail.trim()) {
    params.set("search_email", query.searchEmail.trim());
  }
  if (query.role) {
    params.set("role", query.role);
  }

  return fetchJson<AdminUsersResponse>(`${apiBaseUrl()}/admin/users?${params.toString()}`, {
    headers: authHeaders(),
  });
}

export async function updateUserRoles(userId: string, roles: string[]): Promise<string[]> {
  const payload = await fetchJson<{ roles: string[] }>(`${apiBaseUrl()}/admin/users/${userId}/roles`, {
    method: "PUT",
    headers: authHeaders(true),
    body: JSON.stringify({ roles }),
  });
  return payload.roles;
}

export async function updateUserNames(userId: string, firstName: string | null, lastName: string | null): Promise<{ id: string; first_name: string | null; last_name: string | null }> {
  return fetchJson(`${apiBaseUrl()}/admin/users/${userId}/names`, {
    method: "PUT",
    headers: authHeaders(true),
    body: JSON.stringify({ first_name: firstName, last_name: lastName }),
  });
}
