import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type { EditFieldValue, ProjectEditData, ProjectItem, ProjectLocalizedText, ProjectSavePayload } from "../types";
import { mockProjectsRepository } from "./mockProjectsRepository";
import type { ProjectsRepository } from "./projectsRepository";

type ApiProjectListItem = {
  id: string;
  category_code: string;
  title: string;
  description: string;
  start_utc?: string | null;
  end_utc?: string | null;
  status_type: "waiting" | "ongoing";
  status_text: string;
  category?: { id: string; code: string; icon: string; color: string; label: string };
};

type ApiProjectDetail = ApiProjectListItem & {
  last_modified_at?: string | null;
  last_modified_by?: { id: string; email: string } | null;
};

type ApiEditFieldValue = {
  field_key: string;
  value: string;
  exact_value?: string | null;
  fallback_locale?: string | null;
  fallback_value?: string | null;
};

type ApiProjectEditData = {
  id: string;
  category_id: string;
  start_utc?: string | null;
  end_utc?: string | null;
  status_type: "waiting" | "ongoing";
  locale: string;
  enabled_locales: string[];
  fields: ApiEditFieldValue[];
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
}

function localized(locale: LocaleCode, value: string): ProjectLocalizedText {
  return locale === "en" ? { en: value } : { fr: value };
}

function toProjectItem(locale: LocaleCode, value: ApiProjectListItem | ApiProjectDetail): ProjectItem {
  return {
    id: value.id,
    categoryCode: value.category_code,
    category: value.category,
    title: localized(locale, value.title ?? ""),
    description: localized(locale, value.description ?? ""),
    startUtc: value.start_utc ?? undefined,
    endUtc: value.end_utc ?? undefined,
    statusType: value.status_type,
    statusText: localized(locale, value.status_text ?? ""),
    attachments: [],
    lastModifiedAt: "last_modified_at" in value ? (value.last_modified_at ?? undefined) : undefined,
    lastModifiedBy: "last_modified_by" in value ? (value.last_modified_by ?? null) : undefined,
  };
}

function toEditField(value: ApiEditFieldValue): EditFieldValue {
  return {
    fieldKey: value.field_key,
    value: value.value,
    exactValue: value.exact_value,
    fallbackLocale: value.fallback_locale,
    fallbackValue: value.fallback_value,
  };
}

function toEditData(value: ApiProjectEditData): ProjectEditData {
  return {
    id: value.id,
    categoryId: value.category_id,
    startUtc: value.start_utc ?? undefined,
    endUtc: value.end_utc ?? undefined,
    statusType: value.status_type,
    locale: value.locale,
    enabledLocales: value.enabled_locales,
    fields: value.fields.map(toEditField),
  };
}

function authHeaders(): HeadersInit {
  const token = getAccessToken();
  if (!token && import.meta.env.MODE !== "test") {
    throw new Error("missing access token");
  }
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, { headers: authHeaders() });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchJsonOrNull<T>(url: string): Promise<T | null> {
  const response = await fetch(url, { headers: authHeaders() });
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function sendJson(url: string, method: string, body?: unknown): Promise<void> {
  const response = await fetch(url, {
    method,
    headers: { ...authHeaders(), "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}

function toApiPayload(payload: ProjectSavePayload): Record<string, unknown> {
  return {
    id: payload.id,
    category_id: payload.categoryId,
    start_utc: payload.startUtc ?? null,
    end_utc: payload.endUtc ?? null,
    status_type: payload.statusType,
    locale: payload.locale,
    fields: payload.fields,
  };
}

export class ApiProjectsRepository implements ProjectsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<ProjectItem[]> {
    if (useMockData()) {
      return mockProjectsRepository.list(preferredLanguage, query);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    if (query.trim()) {
      params.set("q", query.trim());
    }
    const payload = await fetchJson<ApiProjectListItem[]>(`${apiBaseUrl()}/projects?${params.toString()}`);
    return payload.map((value) => toProjectItem(preferredLanguage, value));
  }

  async byId(id: string, preferredLanguage: LocaleCode): Promise<ProjectItem | null> {
    if (useMockData()) {
      return mockProjectsRepository.byId(id, preferredLanguage);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiProjectDetail>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}?${params.toString()}`);
    return payload ? toProjectItem(preferredLanguage, payload) : null;
  }

  async editData(id: string, preferredLanguage: LocaleCode): Promise<ProjectEditData | null> {
    if (useMockData()) {
      return mockProjectsRepository.editData(id, preferredLanguage);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiProjectEditData>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}/edit?${params.toString()}`);
    return payload ? toEditData(payload) : null;
  }

  async save(payload: ProjectSavePayload, existingId?: string): Promise<void> {
    if (useMockData()) return;
    const path = existingId ? `/projects/${encodeURIComponent(existingId)}` : "/projects";
    await sendJson(`${apiBaseUrl()}${path}`, existingId ? "PUT" : "POST", toApiPayload(payload));
  }

  async delete(id: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}`, "DELETE");
  }
}

export const apiProjectsRepository = new ApiProjectsRepository();
