import { authenticatedFetch } from "../../auth/authenticatedFetch";

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

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await authenticatedFetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function getToken(): Promise<TokenInfoResponse | null> {
  try {
    return await fetchJson<TokenInfoResponse>(`${apiBaseUrl()}/settings/token`);
  } catch {
    return null;
  }
}

export async function createToken(): Promise<CreateTokenResponse> {
  return fetchJson<CreateTokenResponse>(`${apiBaseUrl()}/settings/token`, {
    method: "POST",
  });
}

export async function revokeToken(): Promise<void> {
  const response = await authenticatedFetch(`${apiBaseUrl()}/settings/token`, {
    method: "DELETE",
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}
