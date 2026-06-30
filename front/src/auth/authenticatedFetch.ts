import { getAccessToken, isAccessTokenUsable, ensureAuthenticated } from "./session";

export async function authenticatedFetch(url: string, options?: RequestInit): Promise<Response> {
  if (!isAccessTokenUsable()) {
    const ok = await ensureAuthenticated();
    if (!ok) {
      redirectToLogin();
      throw new Error("session expired");
    }
  }

  const headers = new Headers(options?.headers);
  headers.set("Authorization", `Bearer ${getAccessToken()}`);

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    redirectToLogin();
    throw new Error("session expired");
  }

  return response;
}

export function currentRedirectPath(): string {
  return window.location.hash.replace(/^#/, "") || "/events";
}

function redirectToLogin(): void {
  window.location.hash = `#/login?redirect=${encodeURIComponent(currentRedirectPath())}`;
}
