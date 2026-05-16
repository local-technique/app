import { sanitizeRedirectPath } from "./redirect";

type Provider = "google" | "facebook";

type SessionPayload = {
  accessToken: string;
  expiresAtUnix?: number;
};

type RefreshResponse = {
  access_token?: string;
  expires_at_unix?: number;
  refresh_token?: string;
};

type ExchangeResponse = {
  access_token?: string;
  expires_at_unix?: number;
  refresh_token?: string;
  redirect?: string;
};

const REFRESH_SKEW_SECONDS = 30;
const REFRESH_TOKEN_STORAGE_KEY = "copro-refresh-token";

let memoryAccessToken = "";
let memoryExpiresAtUnix = 0;

function readRefreshToken(): string {
  try {
    return sessionStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) ?? "";
  } catch {
    return "";
  }
}

function writeRefreshToken(value: string): void {
  try {
    sessionStorage.setItem(REFRESH_TOKEN_STORAGE_KEY, value);
  } catch {}
}

function clearRefreshToken(): void {
  try {
    sessionStorage.removeItem(REFRESH_TOKEN_STORAGE_KEY);
  } catch {}
}

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function parseJwtExp(token: string): number | null {
  const parts = token.split(".");
  if (parts.length !== 3) {
    return null;
  }

  try {
    const base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    const padded = base64.padEnd(Math.ceil(base64.length / 4) * 4, "=");
    const payload = JSON.parse(atob(padded)) as { exp?: number };
    return typeof payload.exp === "number" ? payload.exp : null;
  } catch {
    return null;
  }
}

function nowUnix(): number {
  return Math.floor(Date.now() / 1000);
}

export function setSession(payload: SessionPayload): void {
  memoryAccessToken = payload.accessToken;
  memoryExpiresAtUnix = payload.expiresAtUnix ?? parseJwtExp(payload.accessToken) ?? 0;
}

export function clearSession(): void {
  memoryAccessToken = "";
  memoryExpiresAtUnix = 0;
  clearRefreshToken();
}

export function getAccessToken(): string {
  return memoryAccessToken;
}

export function isAccessTokenUsable(): boolean {
  if (!memoryAccessToken || !memoryExpiresAtUnix) {
    return false;
  }
  return memoryExpiresAtUnix - REFRESH_SKEW_SECONDS > nowUnix();
}

export async function refreshAccessToken(): Promise<boolean> {
  const refreshToken = readRefreshToken();
  if (!refreshToken) {
    clearSession();
    return false;
  }

  try {
    const response = await fetch(`${apiBaseUrl()}/auth/refresh`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    if (!response.ok) {
      clearSession();
      return false;
    }

    const payload = (await response.json()) as RefreshResponse;
    if (!payload.access_token || !payload.refresh_token) {
      clearSession();
      return false;
    }

    setSession({
      accessToken: payload.access_token,
      expiresAtUnix: payload.expires_at_unix,
    });
    writeRefreshToken(payload.refresh_token);

    return true;
  } catch {
    return false;
  }
}

export async function exchangeCallbackCode(code: string): Promise<{ ok: boolean; redirect: string }> {
  try {
    const response = await fetch(`${apiBaseUrl()}/auth/exchange`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ code }),
    });

    if (!response.ok) {
      clearSession();
      return { ok: false, redirect: "/events" };
    }

    const payload = (await response.json()) as ExchangeResponse;
    if (!payload.access_token || !payload.refresh_token) {
      clearSession();
      return { ok: false, redirect: "/events" };
    }

    setSession({
      accessToken: payload.access_token,
      expiresAtUnix: payload.expires_at_unix,
    });
    writeRefreshToken(payload.refresh_token);

    const redirect = typeof payload.redirect === "string" ? sanitizeRedirectPath(payload.redirect) : "/events";
    return { ok: true, redirect };
  } catch {
    clearSession();
    return { ok: false, redirect: "/events" };
  }
}

export async function ensureAuthenticated(): Promise<boolean> {
  if (isAccessTokenUsable()) {
    return true;
  }

  return refreshAccessToken();
}

export function providerStartUrl(provider: Provider, requestedPath: string): string {
  const target = requestedPath || "/events";
  const query = new URLSearchParams({ redirect: target });
  return `${apiBaseUrl()}/auth/${provider}/start?${query.toString()}`;
}

export async function logout(): Promise<void> {
  const token = getAccessToken();
  const refreshToken = readRefreshToken();

  try {
    const headers: Record<string, string> = {};
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }
    headers["Content-Type"] = "application/json";

    await fetch(`${apiBaseUrl()}/auth/logout`, {
      method: "POST",
      headers,
      body: JSON.stringify({ refresh_token: refreshToken || undefined }),
    });
  } finally {
    clearSession();
  }
}
