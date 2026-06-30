import { sanitizeRedirectPath } from "./redirect";
import { reactive } from "vue";

type Provider = "google" | "microsoft";

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

type MeResponse = {
  id?: string;
  roles?: string[];
};

export const currentUserRoles = reactive({
  loaded: false,
  loading: false,
  roles: [] as string[],
});

export const currentUserId = reactive<{ value: string | null }>({ value: null });

const REFRESH_SKEW_SECONDS = 30;
const REFRESH_TOKEN_STORAGE_KEY = "copro-refresh-token";

let memoryAccessToken = "";
let memoryExpiresAtUnix = 0;
let currentUserRolesRequest: Promise<boolean> | null = null;

function readRefreshToken(): string {
  try {
    return localStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) ?? "";
  } catch {
    return "";
  }
}

function writeRefreshToken(value: string): void {
  try {
    localStorage.setItem(REFRESH_TOKEN_STORAGE_KEY, value);
  } catch {}
}

function clearRefreshToken(): void {
  try {
    localStorage.removeItem(REFRESH_TOKEN_STORAGE_KEY);
  } catch {}
}

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
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
  currentUserId.value = null;
  currentUserRoles.loaded = false;
  currentUserRoles.loading = false;
  currentUserRoles.roles = [];
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

let refreshAccessTokenPromise: Promise<boolean> | null = null;

export async function refreshAccessToken(): Promise<boolean> {
  if (refreshAccessTokenPromise) {
    return refreshAccessTokenPromise;
  }

  refreshAccessTokenPromise = (async () => {
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
  })();

  try {
    return await refreshAccessTokenPromise;
  } finally {
    refreshAccessTokenPromise = null;
  }
}

let authInitPromise: Promise<boolean> | null = null;

export function initAuth(): Promise<boolean> {
  if (authInitPromise) {
    return authInitPromise;
  }
  if (useMockData()) {
    authInitPromise = Promise.resolve(true);
    return authInitPromise;
  }
  authInitPromise = ensureAuthenticated();
  return authInitPromise;
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

export function hasRole(role: string): boolean {
  return currentUserRoles.roles.includes(role);
}

export function hasAnyRole(roles: string[]): boolean {
  return roles.some((role) => hasRole(role));
}

export function hasNoRoles(): boolean {
  return currentUserRoles.loaded && currentUserRoles.roles.length === 0;
}

export async function ensureCurrentUserRoles(): Promise<boolean> {
  if (useMockData()) {
    currentUserId.value = "00000000-0000-0000-0000-000000000000";
    currentUserRoles.roles = ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"];
    currentUserRoles.loaded = true;
    return true;
  }

  if (currentUserRoles.loaded) {
    return true;
  }

  if (currentUserRolesRequest) {
    return currentUserRolesRequest;
  }

  currentUserRolesRequest = fetchCurrentUserRoles();
  try {
    return await currentUserRolesRequest;
  } finally {
    currentUserRolesRequest = null;
  }
}

async function fetchCurrentUserRoles(): Promise<boolean> {
  const authenticated = await ensureAuthenticated();
  if (!authenticated) {
    return false;
  }

  currentUserRoles.loading = true;
  try {
    const response = await fetch(`${apiBaseUrl()}/me`, {
      headers: {
        Authorization: `Bearer ${getAccessToken()}`,
      },
    });

    if (!response.ok) {
      if (response.status === 401 || response.status === 403) {
        clearSession();
      }
      return false;
    }

    const payload = (await response.json()) as MeResponse;
    currentUserId.value = payload.id ?? null;
    currentUserRoles.roles = Array.isArray(payload.roles) ? payload.roles : [];
    currentUserRoles.loaded = true;
    return true;
  } catch {
    return false;
  } finally {
    currentUserRoles.loading = false;
  }
}

export async function initOAuthSession(): Promise<void> {
  try {
    const res = await fetch(`${apiBaseUrl()}/auth/session`, {
      method: "GET",
      credentials: "include",
    });
    if (!res.ok) {
      console.warn("oauth session init failed", res.status);
    }
  } catch {
    // session init is best-effort; oauth start will create one if missing
  }
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

export function getCurrentUserId(): string | null {
  return currentUserId.value;
}
