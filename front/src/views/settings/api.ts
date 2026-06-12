import { getAccessToken } from "../../auth/session";

export type TokenInfoResponse = {
  id: string;
  token_prefix: string;
  created_at: string;
  last_used_at: string | null;
};

export type CreateTokenResponse = {
  id: string;
  token_prefix: string;
  token_full: string;
  created_at: string;
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function authHeaders(): HeadersInit {
  const token = getAccessToken();
  if (!token) {
    throw new Error("missing access token");
  }
  return { Authorization: `Bearer ${token}` };
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function getToken(): Promise<TokenInfoResponse | null> {
  try {
    return await fetchJson<TokenInfoResponse>(`${apiBaseUrl()}/settings/token`, {
      headers: authHeaders(),
    });
  } catch {
    return null;
  }
}

export async function createToken(): Promise<CreateTokenResponse> {
  return fetchJson<CreateTokenResponse>(`${apiBaseUrl()}/settings/token`, {
    method: "POST",
    headers: authHeaders(),
  });
}

export async function revokeToken(): Promise<void> {
  await fetch(`${apiBaseUrl()}/settings/token`, {
    method: "DELETE",
    headers: authHeaders(),
  });
}
