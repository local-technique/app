import { getAccessToken } from "../auth/session";
import type { LocaleCode } from "../common/i18n";
import type { CategoryInput, CategoryItem } from "./types";

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function authHeaders(json = false): HeadersInit {
  const token = getAccessToken();
  if (!token && import.meta.env.MODE !== "test") {
    throw new Error("missing access token");
  }
  return {
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(json ? { "Content-Type": "application/json" } : {}),
  };
}

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function listCategories(locale: LocaleCode, admin = false): Promise<CategoryItem[]> {
  if (import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true") {
    return [
      { id: "11111111-1111-4111-8111-111111111111", key: "HEA", icon: "flame", color: "#d73a49", label: locale === "fr" ? "Chauffage" : "Heating", labels: { en: "Heating", fr: "Chauffage" } },
      { id: "22222222-2222-4222-8222-222222222222", key: "ELV", icon: "arrow-up-down", color: "#0366d6", label: locale === "fr" ? "Ascenseur" : "Elevator", labels: { en: "Elevator", fr: "Ascenseur" } },
    ];
  }
  const params = new URLSearchParams({ locale });
  const path = admin ? "/admin/categories" : "/categories";
  return request<CategoryItem[]>(`${apiBaseUrl()}${path}?${params.toString()}`, { headers: authHeaders() });
}

export async function createCategory(input: CategoryInput): Promise<void> {
  const response = await fetch(`${apiBaseUrl()}/admin/categories`, {
    method: "POST",
    headers: authHeaders(true),
    body: JSON.stringify(input),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}

export async function updateCategory(id: string, input: CategoryInput): Promise<void> {
  const response = await fetch(`${apiBaseUrl()}/admin/categories/${encodeURIComponent(id)}`, {
    method: "PUT",
    headers: authHeaders(true),
    body: JSON.stringify(input),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}

export async function deleteCategory(id: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl()}/admin/categories/${encodeURIComponent(id)}`, {
    method: "DELETE",
    headers: authHeaders(),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}
