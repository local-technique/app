import { sanitizeRedirectPath } from "./redirect";

type Provider = "google" | "facebook";

type SessionPayload = {
  accessToken: string;
  expiresAtUnix?: number;
};

type RefreshResponse = {
  access_token?: string;
  expires_at_unix?: number;
};

type ExchangeResponse = {
  access_token?: string;
  expires_at_unix?: number;
  redirect?: string;
};

const REFRESH_SKEW_SECONDS = 30;

let memoryAccessToken = "";
let memoryExpiresAtUnix = 0;

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
  try {
    const response = await fetch(`${apiBaseUrl()}/auth/refresh`, {
      method: "POST",
      credentials: "include",
    });

    if (!response.ok) {
      clearSession();
      return false;
    }

    const payload = (await response.json()) as RefreshResponse;
    if (!payload.access_token) {
      clearSession();
      return false;
    }

    setSession({
      accessToken: payload.access_token,
      expiresAtUnix: payload.expires_at_unix,
    });

    return true;
  } catch {
    return false;
  }
}

export async function exchangeCallbackCode(code: string): Promise<{ ok: boolean; redirect: string }> {
  try {
    const response = await fetch(`${apiBaseUrl()}/auth/exchange`, {
      method: "POST",
      credentials: "include",
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
    if (!payload.access_token) {
      clearSession();
      return { ok: false, redirect: "/events" };
    }

    setSession({
      accessToken: payload.access_token,
      expiresAtUnix: payload.expires_at_unix,
    });

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

  try {
    const headers: Record<string, string> = {};
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    await fetch(`${apiBaseUrl()}/auth/logout`, {
      method: "POST",
      credentials: "include",
      headers,
    });
  } finally {
    clearSession();
  }
}
